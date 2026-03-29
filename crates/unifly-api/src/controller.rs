// ── Controller abstraction ──
//
// Full lifecycle management for a UniFi controller connection.
// Handles authentication, background refresh, command routing,
// and reactive data streaming through the DataStore.

use std::net::Ipv6Addr;
use std::sync::Arc;

use tokio::sync::{Mutex, broadcast, mpsc, watch};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::command::{Command, CommandEnvelope, CommandResult};
use crate::config::{AuthCredentials, ControllerConfig, TlsVerification};
use crate::core_error::CoreError;
use crate::model::{
    AclRule, Admin, Alarm, Client, Country, Device, DnsPolicy, DpiApplication, DpiCategory,
    EntityId, Event, FirewallPolicy, FirewallZone, HealthSummary, MacAddress, Network,
    RadiusProfile, Site, SysInfo, SystemInfo, TrafficMatchingList, Voucher, VpnServer, VpnTunnel,
    WanInterface, WifiBroadcast,
};
use crate::store::{DataStore, event_storage_key};
use crate::stream::EntityStream;

use crate::transport::{TlsMode, TransportConfig};
use crate::websocket::{ReconnectConfig, WebSocketHandle};
use crate::{IntegrationClient, LegacyClient};

mod commands;
mod payloads;
mod query;
mod refresh;

use self::commands::route_command;

const COMMAND_CHANNEL_SIZE: usize = 64;
const EVENT_CHANNEL_SIZE: usize = 256;
const REFRESH_DETAIL_CONCURRENCY: usize = 16;

// ── ConnectionState ──────────────────────────────────────────────

/// Connection state observable by consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
    Failed,
}

// ── Controller ───────────────────────────────────────────────────

/// The main entry point for consumers.
///
/// Cheaply cloneable via `Arc<ControllerInner>`. Manages the full
/// connection lifecycle: authentication, background data refresh,
/// command routing, and reactive entity streaming.
#[derive(Clone)]
pub struct Controller {
    inner: Arc<ControllerInner>,
}

struct ControllerInner {
    config: ControllerConfig,
    store: Arc<DataStore>,
    connection_state: watch::Sender<ConnectionState>,
    event_tx: broadcast::Sender<Arc<Event>>,
    command_tx: Mutex<mpsc::Sender<CommandEnvelope>>,
    command_rx: Mutex<Option<mpsc::Receiver<CommandEnvelope>>>,
    cancel: CancellationToken,
    /// Child token for the current connection — cancelled on disconnect,
    /// replaced on reconnect (avoids permanent cancellation).
    cancel_child: Mutex<CancellationToken>,
    legacy_client: Mutex<Option<Arc<LegacyClient>>>,
    integration_client: Mutex<Option<Arc<IntegrationClient>>>,
    /// Resolved Integration API site UUID (populated on connect).
    site_id: Mutex<Option<uuid::Uuid>>,
    /// WebSocket event stream handle (populated on connect if enabled).
    ws_handle: Mutex<Option<WebSocketHandle>>,
    task_handles: Mutex<Vec<JoinHandle<()>>>,
    /// Warnings accumulated during connect (e.g. Legacy auth failure in Hybrid mode).
    warnings: Mutex<Vec<String>>,
}

impl Controller {
    /// Create a new Controller from configuration. Does NOT connect --
    /// call [`connect()`](Self::connect) to authenticate and start background tasks.
    pub fn new(config: ControllerConfig) -> Self {
        let store = Arc::new(DataStore::new());
        let (connection_state, _) = watch::channel(ConnectionState::Disconnected);
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_SIZE);
        let (command_tx, command_rx) = mpsc::channel(COMMAND_CHANNEL_SIZE);
        let cancel = CancellationToken::new();
        let cancel_child = cancel.child_token();

        Self {
            inner: Arc::new(ControllerInner {
                config,
                store,
                connection_state,
                event_tx,
                command_tx: Mutex::new(command_tx),
                command_rx: Mutex::new(Some(command_rx)),
                cancel,
                cancel_child: Mutex::new(cancel_child),
                legacy_client: Mutex::new(None),
                integration_client: Mutex::new(None),
                warnings: Mutex::new(Vec::new()),
                site_id: Mutex::new(None),
                ws_handle: Mutex::new(None),
                task_handles: Mutex::new(Vec::new()),
            }),
        }
    }

    /// Access the controller configuration.
    pub fn config(&self) -> &ControllerConfig {
        &self.inner.config
    }

    /// Access the underlying DataStore.
    pub fn store(&self) -> &Arc<DataStore> {
        &self.inner.store
    }

    // ── Connection lifecycle ─────────────────────────────────────

    /// Connect to the controller.
    ///
    /// Detects the platform, authenticates, performs an initial data
    /// refresh, and spawns background tasks (periodic refresh, command
    /// processor).
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    pub async fn connect(&self) -> Result<(), CoreError> {
        let _ = self
            .inner
            .connection_state
            .send(ConnectionState::Connecting);

        // Fresh child token for this connection (supports reconnect).
        let child = self.inner.cancel.child_token();
        *self.inner.cancel_child.lock().await = child.clone();

        let config = &self.inner.config;
        let transport = build_transport(config);

        match &config.auth {
            AuthCredentials::ApiKey(api_key) => {
                // Detect platform so we use the right URL prefix
                let platform = LegacyClient::detect_platform(&config.url).await?;
                debug!(?platform, "detected controller platform");

                // Integration API client (preferred)
                let integration = IntegrationClient::from_api_key(
                    config.url.as_str(),
                    api_key,
                    &transport,
                    platform,
                )?;

                // Resolve site UUID from Integration API
                let site_id = resolve_site_id(&integration, &config.site).await?;
                debug!(site_id = %site_id, "resolved Integration API site UUID");

                *self.inner.integration_client.lock().await = Some(Arc::new(integration));
                *self.inner.site_id.lock().await = Some(site_id);
            }
            AuthCredentials::Credentials { username, password } => {
                // Legacy-only auth
                let platform = LegacyClient::detect_platform(&config.url).await?;
                debug!(?platform, "detected controller platform");

                let client = LegacyClient::new(
                    config.url.clone(),
                    config.site.clone(),
                    platform,
                    &transport,
                )?;
                client.login(username, password).await?;
                debug!("session authentication successful");

                *self.inner.legacy_client.lock().await = Some(Arc::new(client));
            }
            AuthCredentials::Hybrid {
                api_key,
                username,
                password,
            } => {
                // Hybrid: both Integration API (API key) and Legacy API (session auth)
                let platform = LegacyClient::detect_platform(&config.url).await?;
                debug!(?platform, "detected controller platform (hybrid)");

                // Integration API client
                let integration = IntegrationClient::from_api_key(
                    config.url.as_str(),
                    api_key,
                    &transport,
                    platform,
                )?;

                let site_id = resolve_site_id(&integration, &config.site).await?;
                debug!(site_id = %site_id, "resolved Integration API site UUID");

                *self.inner.integration_client.lock().await = Some(Arc::new(integration));
                *self.inner.site_id.lock().await = Some(site_id);

                // Legacy API client — attempt login but degrade gracefully
                // if it fails. The Integration API is the primary surface;
                // Legacy adds events, stats, and admin ops.
                match LegacyClient::new(
                    config.url.clone(),
                    config.site.clone(),
                    platform,
                    &transport,
                ) {
                    Ok(client) => match client.login(username, password).await {
                        Ok(()) => {
                            debug!("legacy session authentication successful (hybrid)");
                            *self.inner.legacy_client.lock().await = Some(Arc::new(client));
                        }
                        Err(e) => {
                            let msg = format!(
                                "Legacy login failed: {e} — events, health stats, and client traffic will be unavailable"
                            );
                            warn!("{msg}");
                            self.inner.warnings.lock().await.push(msg);
                        }
                    },
                    Err(e) => {
                        let msg = format!("Legacy client setup failed: {e}");
                        warn!("{msg}");
                        self.inner.warnings.lock().await.push(msg);
                    }
                }
            }
            AuthCredentials::Cloud { api_key, host_id } => {
                let integration = IntegrationClient::from_api_key(
                    config.url.as_str(),
                    api_key,
                    &transport,
                    crate::ControllerPlatform::Cloud,
                )?;

                let site_id = if let Ok(uuid) = uuid::Uuid::parse_str(&config.site) {
                    uuid
                } else if let Ok(uuid) = uuid::Uuid::parse_str(host_id) {
                    uuid
                } else {
                    resolve_site_id(&integration, &config.site).await?
                };
                debug!(site_id = %site_id, "resolved cloud Integration API site UUID");

                *self.inner.integration_client.lock().await = Some(Arc::new(integration));
                *self.inner.site_id.lock().await = Some(site_id);

                let msg =
                    "Cloud auth mode active: Legacy API and WebSocket features are unavailable"
                        .to_string();
                self.inner.warnings.lock().await.push(msg);
            }
        }

        // Initial data load
        self.full_refresh().await?;

        // Spawn background tasks
        let mut handles = self.inner.task_handles.lock().await;

        if let Some(rx) = self.inner.command_rx.lock().await.take() {
            let ctrl = self.clone();
            handles.push(tokio::spawn(command_processor_task(ctrl, rx)));
        }

        let interval_secs = config.refresh_interval_secs;
        if interval_secs > 0 {
            let ctrl = self.clone();
            let cancel = child.clone();
            handles.push(tokio::spawn(refresh::refresh_task(
                ctrl,
                interval_secs,
                cancel,
            )));
        }

        // WebSocket event stream
        if config.websocket_enabled {
            self.spawn_websocket(&child, &mut handles).await;
        }

        let _ = self.inner.connection_state.send(ConnectionState::Connected);
        info!("connected to controller");
        Ok(())
    }

    /// Spawn the WebSocket event stream and a bridge task that converts
    /// raw [`UnifiEvent`]s into domain [`Event`]s and broadcasts them.
    ///
    /// Non-fatal on failure — the TUI falls back to polling.
    async fn spawn_websocket(&self, cancel: &CancellationToken, handles: &mut Vec<JoinHandle<()>>) {
        let Some(legacy) = self.inner.legacy_client.lock().await.clone() else {
            debug!("no legacy client — WebSocket unavailable");
            return;
        };

        let platform = legacy.platform();
        let Some(ws_path_template) = platform.websocket_path() else {
            debug!("platform does not support WebSocket");
            return;
        };

        let ws_path = ws_path_template.replace("{site}", &self.inner.config.site);
        let base_url = &self.inner.config.url;
        let scheme = if base_url.scheme() == "https" {
            "wss"
        } else {
            "ws"
        };
        let host = base_url.host_str().unwrap_or("localhost");
        let ws_url_str = match base_url.port() {
            Some(p) => format!("{scheme}://{host}:{p}{ws_path}"),
            None => format!("{scheme}://{host}{ws_path}"),
        };
        let ws_url = match url::Url::parse(&ws_url_str) {
            Ok(u) => u,
            Err(e) => {
                warn!(error = %e, url = %ws_url_str, "invalid WebSocket URL");
                return;
            }
        };

        let cookie = legacy.cookie_header();

        if cookie.is_none() {
            warn!("no session cookie — WebSocket requires legacy auth (skipping)");
            return;
        }

        let ws_tls = tls_to_transport(&self.inner.config.tls);
        let ws_cancel = cancel.child_token();
        let handle = match WebSocketHandle::connect(
            ws_url,
            ReconnectConfig::default(),
            ws_cancel.clone(),
            cookie,
            ws_tls,
        ) {
            Ok(h) => h,
            Err(e) => {
                warn!(error = %e, "WebSocket connection failed (non-fatal)");
                return;
            }
        };

        // Bridge task: WS events → domain Events → broadcast channel.
        // Also extracts real-time device stats from `device:sync` messages
        // to feed the dashboard chart without waiting for full_refresh().
        let mut ws_rx = handle.subscribe();
        let event_tx = self.inner.event_tx.clone();
        let store = Arc::clone(&self.inner.store);
        let bridge_cancel = ws_cancel;

        handles.push(tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    () = bridge_cancel.cancelled() => break,
                    result = ws_rx.recv() => {
                        match result {
                            Ok(ws_event) => {
                                store.mark_ws_event(chrono::Utc::now());

                                // Extract real-time stats from device:sync messages
                                if ws_event.key == "device:sync" || ws_event.key == "device:update" {
                                    apply_device_sync(&store, &ws_event.extra);
                                }

                                // Only broadcast actual events (key starts with EVT_),
                                // not sync/state-dump messages.
                                if ws_event.key.starts_with("EVT_") {
                                    let event = crate::model::event::Event::from(
                                        (*ws_event).clone(),
                                    );
                                    let _ = event_tx.send(Arc::new(event));
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                                warn!(skipped = n, "WS bridge: receiver lagged");
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        }));

        *self.inner.ws_handle.lock().await = Some(handle);
        info!("WebSocket event stream spawned (handshake in progress)");
    }

    /// Disconnect from the controller.
    ///
    /// Cancels background tasks, logs out if session-based, and resets
    /// the connection state to [`Disconnected`](ConnectionState::Disconnected).
    pub async fn disconnect(&self) {
        // Cancel the child token (not the parent — allows reconnect).
        self.inner.cancel_child.lock().await.cancel();

        // Shut down WebSocket promptly so any active read/handshake wakes up.
        if let Some(handle) = self.inner.ws_handle.lock().await.take() {
            handle.shutdown();
        }

        // Join all background tasks
        let mut handles = self.inner.task_handles.lock().await;
        for handle in handles.drain(..) {
            let _ = handle.await;
        }

        let legacy = self.inner.legacy_client.lock().await.clone();

        // Logout if session-based (Credentials or Hybrid both have active sessions)
        if matches!(
            self.inner.config.auth,
            AuthCredentials::Credentials { .. } | AuthCredentials::Hybrid { .. }
        ) && let Some(client) = legacy
            && let Err(e) = client.logout().await
        {
            warn!(error = %e, "logout failed (non-fatal)");
        }

        *self.inner.legacy_client.lock().await = None;
        *self.inner.integration_client.lock().await = None;
        *self.inner.site_id.lock().await = None;

        // Recreate command channel so reconnects can spawn a fresh receiver.
        // The previous receiver is consumed by the command processor task.
        {
            let (tx, rx) = mpsc::channel(COMMAND_CHANNEL_SIZE);
            *self.inner.command_tx.lock().await = tx;
            *self.inner.command_rx.lock().await = Some(rx);
        }

        let _ = self
            .inner
            .connection_state
            .send(ConnectionState::Disconnected);
        debug!("disconnected");
    }

    // ── Command execution ────────────────────────────────────────

    /// Execute a command against the controller.
    ///
    /// Sends the command through the internal channel to the command
    /// processor task and awaits the result.
    pub async fn execute(&self, cmd: Command) -> Result<CommandResult, CoreError> {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let command_tx = self.inner.command_tx.lock().await.clone();

        command_tx
            .send(CommandEnvelope {
                command: cmd,
                response_tx: tx,
            })
            .await
            .map_err(|_| CoreError::ControllerDisconnected)?;

        rx.await.map_err(|_| CoreError::ControllerDisconnected)?
    }

    // ── One-shot convenience ─────────────────────────────────────

    /// One-shot: connect, run closure, disconnect.
    ///
    /// Optimized for CLI: disables WebSocket and periodic refresh since
    /// we only need a single request-response cycle.
    pub async fn oneshot<F, Fut, T>(config: ControllerConfig, f: F) -> Result<T, CoreError>
    where
        F: FnOnce(Controller) -> Fut,
        Fut: std::future::Future<Output = Result<T, CoreError>>,
    {
        let mut cfg = config;
        cfg.websocket_enabled = false;
        cfg.refresh_interval_secs = 0;

        let controller = Controller::new(cfg);
        controller.connect().await?;
        let result = f(controller.clone()).await;
        controller.disconnect().await;
        result
    }
}

// ── Background tasks ─────────────────────────────────────────────

/// Parse a numeric field from a JSON object, tolerating both string and number encodings.
fn parse_f64_field(parent: Option<&serde_json::Value>, key: &str) -> Option<f64> {
    parent.and_then(|s| s.get(key)).and_then(|v| {
        v.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| v.as_f64())
    })
}

/// Apply a `device:sync` WebSocket message to the DataStore.
///
/// Extracts CPU, memory, load averages, and uplink bandwidth from the
/// raw Legacy API device JSON. Merges stats into the existing device
/// (looked up by MAC) without clobbering Integration API fields.
#[allow(clippy::cast_precision_loss)]
fn apply_device_sync(store: &DataStore, data: &serde_json::Value) {
    let Some(mac_str) = data.get("mac").and_then(serde_json::Value::as_str) else {
        return;
    };
    let mac = MacAddress::new(mac_str);
    let Some(existing) = store.device_by_mac(&mac) else {
        return; // Device not in store yet — full_refresh will add it
    };

    // Parse sys_stats
    let sys = data.get("sys_stats");
    let cpu = sys
        .and_then(|s| s.get("cpu"))
        .and_then(|v| v.as_str().or_else(|| v.as_f64().map(|_| "")))
        .and_then(|s| {
            if s.is_empty() {
                None
            } else {
                s.parse::<f64>().ok()
            }
        })
        .or_else(|| {
            sys.and_then(|s| s.get("cpu"))
                .and_then(serde_json::Value::as_f64)
        });
    #[allow(clippy::as_conversions, clippy::cast_precision_loss)]
    let mem_pct = match (
        sys.and_then(|s| s.get("mem_used"))
            .and_then(serde_json::Value::as_i64),
        sys.and_then(|s| s.get("mem_total"))
            .and_then(serde_json::Value::as_i64),
    ) {
        (Some(used), Some(total)) if total > 0 => Some((used as f64 / total as f64) * 100.0),
        _ => None,
    };
    let load_averages: [Option<f64>; 3] =
        ["loadavg_1", "loadavg_5", "loadavg_15"].map(|key| parse_f64_field(sys, key));

    // Uplink bandwidth: check "uplink" object or top-level fields
    let uplink = data.get("uplink");
    let tx_bps = uplink
        .and_then(|u| u.get("tx_bytes-r").or_else(|| u.get("tx_bytes_r")))
        .and_then(serde_json::Value::as_u64)
        .or_else(|| data.get("tx_bytes-r").and_then(serde_json::Value::as_u64));
    let rx_bps = uplink
        .and_then(|u| u.get("rx_bytes-r").or_else(|| u.get("rx_bytes_r")))
        .and_then(serde_json::Value::as_u64)
        .or_else(|| data.get("rx_bytes-r").and_then(serde_json::Value::as_u64));

    let bandwidth = match (tx_bps, rx_bps) {
        (Some(tx), Some(rx)) if tx > 0 || rx > 0 => Some(crate::model::common::Bandwidth {
            tx_bytes_per_sec: tx,
            rx_bytes_per_sec: rx,
        }),
        _ => existing.stats.uplink_bandwidth, // Keep existing if no new data
    };

    // Uptime from top-level `_uptime` or `uptime`
    let uptime = data
        .get("_uptime")
        .or_else(|| data.get("uptime"))
        .and_then(serde_json::Value::as_i64)
        .and_then(|u| u.try_into().ok())
        .or(existing.stats.uptime_secs);

    // Clone and update
    let mut device = (*existing).clone();
    device.stats.uplink_bandwidth = bandwidth;
    if let Some(c) = cpu {
        device.stats.cpu_utilization_pct = Some(c);
    }
    if let Some(m) = mem_pct {
        device.stats.memory_utilization_pct = Some(m);
    }
    if let Some(l) = load_averages[0] {
        device.stats.load_average_1m = Some(l);
    }
    if let Some(l) = load_averages[1] {
        device.stats.load_average_5m = Some(l);
    }
    if let Some(l) = load_averages[2] {
        device.stats.load_average_15m = Some(l);
    }
    device.stats.uptime_secs = uptime;

    // Update client count from num_sta (AP/switch connected stations)
    if let Some(num_sta) = data.get("num_sta").and_then(serde_json::Value::as_u64) {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        {
            device.client_count = Some(num_sta as u32);
        }
    }

    if let Some(obj) = data.as_object()
        && let Some(wan_ipv6) = parse_legacy_device_wan_ipv6(obj)
    {
        device.wan_ipv6 = Some(wan_ipv6);
    }

    let key = mac.as_str().to_owned();
    let id = device.id.clone();
    store.devices.upsert(key, id, device);
}

/// Process commands from the mpsc channel, routing each to the
/// appropriate Legacy API call.
async fn command_processor_task(controller: Controller, mut rx: mpsc::Receiver<CommandEnvelope>) {
    let cancel = controller.inner.cancel_child.lock().await.clone();

    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => break,
            envelope = rx.recv() => {
                let Some(envelope) = envelope else { break };
                let result = route_command(&controller, envelope.command).await;
                let _ = envelope.response_tx.send(result);
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn parse_ipv6_text(raw: &str) -> Option<Ipv6Addr> {
    let candidate = raw.trim().split('/').next().unwrap_or(raw).trim();
    candidate.parse::<Ipv6Addr>().ok()
}

fn pick_ipv6_from_value(value: &serde_json::Value) -> Option<String> {
    let mut first_link_local: Option<String> = None;

    let iter: Box<dyn Iterator<Item = &serde_json::Value> + '_> = match value {
        serde_json::Value::Array(items) => Box::new(items.iter()),
        _ => Box::new(std::iter::once(value)),
    };

    for item in iter {
        if let Some(ipv6) = item.as_str().and_then(parse_ipv6_text) {
            let ip_text = ipv6.to_string();
            if !ipv6.is_unicast_link_local() {
                return Some(ip_text);
            }
            if first_link_local.is_none() {
                first_link_local = Some(ip_text);
            }
        }
    }

    first_link_local
}

fn parse_legacy_device_wan_ipv6(
    extra: &serde_json::Map<String, serde_json::Value>,
) -> Option<String> {
    // Primary source on gateways: wan1.ipv6 = ["global", "link-local"].
    if let Some(v) = extra
        .get("wan1")
        .and_then(|wan| wan.get("ipv6"))
        .and_then(pick_ipv6_from_value)
    {
        return Some(v);
    }

    // Fallback source on some firmware: top-level ipv6 array.
    extra.get("ipv6").and_then(pick_ipv6_from_value)
}

/// Convert raw health JSON values into domain `HealthSummary` types.
fn convert_health_summaries(raw: Vec<serde_json::Value>) -> Vec<HealthSummary> {
    raw.into_iter()
        .map(|v| HealthSummary {
            subsystem: v
                .get("subsystem")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            status: v
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            num_adopted: v
                .get("num_adopted")
                .and_then(serde_json::Value::as_u64)
                .map(|n| n as u32),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            num_sta: v
                .get("num_sta")
                .and_then(serde_json::Value::as_u64)
                .map(|n| n as u32),
            tx_bytes_r: v.get("tx_bytes-r").and_then(serde_json::Value::as_u64),
            rx_bytes_r: v.get("rx_bytes-r").and_then(serde_json::Value::as_u64),
            latency: v.get("latency").and_then(serde_json::Value::as_f64),
            wan_ip: v.get("wan_ip").and_then(|v| v.as_str()).map(String::from),
            gateways: v.get("gateways").and_then(|v| v.as_array()).map(|a| {
                a.iter()
                    .filter_map(|g| g.as_str().map(String::from))
                    .collect()
            }),
            extra: v,
        })
        .collect()
}

/// Build a [`TransportConfig`] from the controller configuration.
fn build_transport(config: &ControllerConfig) -> TransportConfig {
    TransportConfig {
        tls: tls_to_transport(&config.tls),
        timeout: config.timeout,
        cookie_jar: None, // LegacyClient::new adds one automatically
    }
}

fn tls_to_transport(tls: &TlsVerification) -> TlsMode {
    match tls {
        TlsVerification::SystemDefaults => TlsMode::System,
        TlsVerification::CustomCa(path) => TlsMode::CustomCa(path.clone()),
        TlsVerification::DangerAcceptInvalid => TlsMode::DangerAcceptInvalid,
    }
}

/// Resolve the Integration API site UUID from a site name or UUID string.
///
/// If `site_name` is already a valid UUID, returns it directly.
/// Otherwise lists all sites and finds the one matching by `internal_reference`.
async fn resolve_site_id(
    client: &IntegrationClient,
    site_name: &str,
) -> Result<uuid::Uuid, CoreError> {
    // Fast path: if the input is already a UUID, use it directly.
    if let Ok(uuid) = uuid::Uuid::parse_str(site_name) {
        return Ok(uuid);
    }

    let sites = client
        .paginate_all(50, |off, lim| client.list_sites(off, lim))
        .await?;

    sites
        .into_iter()
        .find(|s| s.internal_reference == site_name)
        .map(|s| s.id)
        .ok_or_else(|| CoreError::SiteNotFound {
            name: site_name.to_owned(),
        })
}

/// Extract a `Uuid` from an `EntityId`, or return an error.
fn require_uuid(id: &EntityId) -> Result<uuid::Uuid, CoreError> {
    id.as_uuid().copied().ok_or_else(|| CoreError::Unsupported {
        operation: "Integration API operation on legacy ID".into(),
        required: "UUID-based entity ID".into(),
    })
}

fn require_legacy<'a>(
    legacy: Option<&'a Arc<LegacyClient>>,
) -> Result<&'a LegacyClient, CoreError> {
    legacy
        .map(Arc::as_ref)
        .ok_or_else(|| CoreError::Unsupported {
            operation: "Legacy API operation".into(),
            required: "Legacy API credentials".into(),
        })
}

fn require_integration<'a>(
    integration: Option<&'a Arc<IntegrationClient>>,
    site_id: Option<uuid::Uuid>,
    operation: &str,
) -> Result<(&'a IntegrationClient, uuid::Uuid), CoreError> {
    let client = integration
        .map(Arc::as_ref)
        .ok_or_else(|| unsupported(operation))?;
    let sid = site_id.ok_or_else(|| unsupported(operation))?;
    Ok((client, sid))
}

fn unsupported(operation: &str) -> CoreError {
    CoreError::Unsupported {
        operation: operation.into(),
        required: "Integration API".into(),
    }
}

/// Resolve an [`EntityId`] to a device MAC via the DataStore.
fn device_mac(store: &DataStore, id: &EntityId) -> Result<MacAddress, CoreError> {
    store
        .device_by_id(id)
        .map(|d| d.mac.clone())
        .ok_or_else(|| CoreError::DeviceNotFound {
            identifier: id.to_string(),
        })
}

/// Resolve an [`EntityId`] to a client MAC via the DataStore.
fn client_mac(store: &DataStore, id: &EntityId) -> Result<MacAddress, CoreError> {
    store
        .client_by_id(id)
        .map(|c| c.mac.clone())
        .ok_or_else(|| CoreError::ClientNotFound {
            identifier: id.to_string(),
        })
}
