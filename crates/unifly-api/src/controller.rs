// ── Controller abstraction ──
//
// Full lifecycle management for a UniFi controller connection.
// Handles authentication, background refresh, command routing,
// and reactive data streaming through the DataStore.

use std::collections::{HashMap, HashSet};
use std::net::Ipv6Addr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{self, StreamExt};
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
            handles.push(tokio::spawn(refresh_task(ctrl, interval_secs, cancel)));
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

    /// Fetch all data from the controller and update the DataStore.
    ///
    /// Pulls devices, clients, and events from the Legacy API, converts
    /// them to domain types, and applies them to the store. Events are
    /// broadcast through the event channel (not stored).
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    pub async fn full_refresh(&self) -> Result<(), CoreError> {
        let integration = self.inner.integration_client.lock().await.clone();
        let site_id = *self.inner.site_id.lock().await;

        if let (Some(integration), Some(sid)) = (integration, site_id) {
            // ── Integration API path (preferred) ─────────────────
            let page_limit = 200;

            let (devices_res, clients_res, networks_res, wifi_res) = tokio::join!(
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_devices(&sid, off, lim)
                }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_clients(&sid, off, lim)
                }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_networks(&sid, off, lim)
                }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_wifi_broadcasts(&sid, off, lim)
                }),
            );

            let (policies_res, zones_res, acls_res, dns_res, vouchers_res) = tokio::join!(
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_firewall_policies(&sid, off, lim)
                }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_firewall_zones(&sid, off, lim)
                }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_acl_rules(&sid, off, lim)
                }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_dns_policies(&sid, off, lim)
                }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_vouchers(&sid, off, lim)
                }),
            );

            let (sites_res, tml_res) = tokio::join!(
                integration.paginate_all(50, |off, lim| { integration.list_sites(off, lim) }),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_traffic_matching_lists(&sid, off, lim)
                }),
            );

            // Core endpoints — failure is fatal
            let devices: Vec<Device> = devices_res?.into_iter().map(Device::from).collect();
            let mut clients: Vec<Client> = clients_res?.into_iter().map(Client::from).collect();
            // Fetch full details for each network (list endpoint omits ipv4/ipv6 config)
            let network_ids: Vec<uuid::Uuid> = networks_res?.into_iter().map(|n| n.id).collect();
            info!(
                network_count = network_ids.len(),
                "fetching network details"
            );
            let networks: Vec<Network> = {
                stream::iter(network_ids.into_iter().map(|nid| {
                    let integration = Arc::clone(&integration);
                    async move {
                        match integration.get_network(&sid, &nid).await {
                            Ok(detail) => Some(Network::from(detail)),
                            Err(e) => {
                                warn!(network_id = %nid, error = %e, "network detail fetch failed");
                                None
                            }
                        }
                    }
                }))
                .buffer_unordered(REFRESH_DETAIL_CONCURRENCY)
                .filter_map(async move |network| network)
                .collect::<Vec<_>>()
                .await
            };
            let wifi: Vec<WifiBroadcast> = wifi_res?.into_iter().map(WifiBroadcast::from).collect();
            let policies: Vec<FirewallPolicy> = policies_res?
                .into_iter()
                .map(FirewallPolicy::from)
                .collect();
            let zones: Vec<FirewallZone> = zones_res?.into_iter().map(FirewallZone::from).collect();
            let sites: Vec<Site> = sites_res?.into_iter().map(Site::from).collect();
            let traffic_matching_lists: Vec<TrafficMatchingList> = tml_res?
                .into_iter()
                .map(TrafficMatchingList::from)
                .collect();

            // Optional endpoints — 404 means the controller doesn't support them
            let acls: Vec<AclRule> = unwrap_or_empty("acl/rules", acls_res);
            let dns: Vec<DnsPolicy> = unwrap_or_empty("dns/policies", dns_res);
            let vouchers: Vec<Voucher> = unwrap_or_empty("vouchers", vouchers_res);

            // Enrich devices with per-device statistics (parallel, non-fatal)
            info!(
                device_count = devices.len(),
                "enriching devices with statistics"
            );
            let mut devices = {
                stream::iter(devices.into_iter().map(|mut device| {
                    let integration = Arc::clone(&integration);
                    async move {
                        if let EntityId::Uuid(device_uuid) = &device.id {
                            match integration.get_device_statistics(&sid, device_uuid).await {
                                Ok(stats_resp) => {
                                    device.stats =
                                        crate::convert::device_stats_from_integration(&stats_resp);
                                }
                                Err(e) => {
                                    warn!(
                                        device = ?device.name,
                                        error = %e,
                                        "device stats fetch failed"
                                    );
                                }
                            }
                        }
                        device
                    }
                }))
                .buffer_unordered(REFRESH_DETAIL_CONCURRENCY)
                .collect::<Vec<_>>()
                .await
            };

            // Supplement with Legacy API data (events, health, client traffic, device stats, DHCP reservations)
            #[allow(clippy::type_complexity)]
            let (legacy_events, legacy_health, legacy_clients, legacy_devices, legacy_users): (
                Vec<Event>,
                Vec<HealthSummary>,
                Vec<crate::legacy::models::LegacyClientEntry>,
                Vec<crate::legacy::models::LegacyDevice>,
                Vec<crate::legacy::models::LegacyUserEntry>,
            ) = match self.inner.legacy_client.lock().await.clone() {
                Some(legacy) => {
                    let (events_res, health_res, clients_res, devices_res, users_res) =
                        tokio::join!(
                            legacy.list_events(Some(100)),
                            legacy.get_health(),
                            legacy.list_clients(),
                            legacy.list_devices(),
                            legacy.list_users(),
                        );

                    let events = match events_res {
                        Ok(raw) => raw.into_iter().map(Event::from).collect(),
                        Err(e) => {
                            warn!(error = %e, "legacy event fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    let health = match health_res {
                        Ok(raw) => convert_health_summaries(raw),
                        Err(e) => {
                            warn!(error = %e, "legacy health fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    let lc = match clients_res {
                        Ok(raw) => raw,
                        Err(e) => {
                            warn!(
                                error = %e,
                                "legacy client fetch failed (non-fatal)"
                            );
                            Vec::new()
                        }
                    };

                    let ld = match devices_res {
                        Ok(raw) => raw,
                        Err(e) => {
                            warn!(error = %e, "legacy device fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    let lu = match users_res {
                        Ok(raw) => raw,
                        Err(e) => {
                            warn!(error = %e, "legacy user fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    (events, health, lc, ld, lu)
                }
                None => (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            };

            // Merge Legacy client traffic (tx/rx bytes, hostname) into Integration clients.
            // Match by IP address — Integration API clients often lack real MAC addresses
            // in the access object, falling back to UUIDs which don't match Legacy MACs.
            if !legacy_clients.is_empty() {
                let legacy_by_ip: HashMap<&str, &crate::legacy::models::LegacyClientEntry> =
                    legacy_clients
                        .iter()
                        .filter_map(|lc| lc.ip.as_deref().map(|ip| (ip, lc)))
                        .collect();
                let mut merged = 0u32;
                for client in &mut clients {
                    let ip_key = client.ip.map(|ip| ip.to_string());
                    if let Some(lc) = ip_key.as_deref().and_then(|ip| legacy_by_ip.get(ip)) {
                        if client.tx_bytes.is_none() {
                            client.tx_bytes = lc.tx_bytes.and_then(|b| u64::try_from(b).ok());
                        }
                        if client.rx_bytes.is_none() {
                            client.rx_bytes = lc.rx_bytes.and_then(|b| u64::try_from(b).ok());
                        }
                        if client.hostname.is_none() {
                            client.hostname.clone_from(&lc.hostname);
                        }
                        // Merge wireless info (Legacy has AP MAC, signal, channel)
                        if client.wireless.is_none() {
                            let legacy_client: Client = Client::from((*lc).clone());
                            client.wireless = legacy_client.wireless;
                            if client.uplink_device_mac.is_none() {
                                client.uplink_device_mac = legacy_client.uplink_device_mac;
                            }
                        }
                        merged += 1;
                    }
                }
                debug!(
                    total_clients = clients.len(),
                    legacy_available = legacy_by_ip.len(),
                    merged,
                    "client traffic merge (by IP)"
                );
            }

            // Merge Legacy user DHCP reservations (fixed IP) into clients by MAC.
            if !legacy_users.is_empty() {
                let users_by_mac: HashMap<String, &crate::legacy::models::LegacyUserEntry> =
                    legacy_users
                        .iter()
                        .map(|u| (u.mac.to_lowercase(), u))
                        .collect();
                for client in &mut clients {
                    if let Some(user) = users_by_mac.get(&client.mac.as_str().to_lowercase()) {
                        client.use_fixedip = user.use_fixedip.unwrap_or(false);
                        client.fixed_ip = user.fixed_ip.as_deref().and_then(|s| s.parse().ok());
                    }
                }
            }

            // Merge Legacy device num_sta (client counts) into Integration devices
            if !legacy_devices.is_empty() {
                let legacy_by_mac: HashMap<&str, &crate::legacy::models::LegacyDevice> =
                    legacy_devices.iter().map(|d| (d.mac.as_str(), d)).collect();
                for device in &mut devices {
                    if let Some(ld) = legacy_by_mac.get(device.mac.as_str()) {
                        if device.client_count.is_none() {
                            device.client_count = ld.num_sta.and_then(|n| n.try_into().ok());
                        }
                        if device.wan_ipv6.is_none() {
                            device.wan_ipv6 = parse_legacy_device_wan_ipv6(&ld.extra);
                        }
                    }
                }
            }

            // Push health to DataStore
            if !legacy_health.is_empty() {
                self.inner
                    .store
                    .site_health
                    .send_modify(|h| *h = Arc::new(legacy_health));
            }

            let fresh_legacy_events = unseen_events(self.store(), &legacy_events);

            self.inner
                .store
                .apply_integration_snapshot(crate::store::RefreshSnapshot {
                    devices,
                    clients,
                    networks,
                    wifi,
                    policies,
                    zones,
                    acls,
                    dns,
                    vouchers,
                    sites,
                    events: legacy_events,
                    traffic_matching_lists,
                });

            for event in fresh_legacy_events {
                let _ = self.inner.event_tx.send(Arc::new(event));
            }
        } else {
            // ── Legacy-only path ─────────────────────────────────
            let legacy = self
                .inner
                .legacy_client
                .lock()
                .await
                .clone()
                .ok_or(CoreError::ControllerDisconnected)?;

            let (devices_res, clients_res, events_res, sites_res) = tokio::join!(
                legacy.list_devices(),
                legacy.list_clients(),
                legacy.list_events(Some(100)),
                legacy.list_sites(),
            );

            let devices: Vec<Device> = devices_res?.into_iter().map(Device::from).collect();
            let clients: Vec<Client> = clients_res?.into_iter().map(Client::from).collect();
            let events: Vec<Event> = events_res?.into_iter().map(Event::from).collect();
            let sites: Vec<Site> = sites_res?.into_iter().map(Site::from).collect();
            let fresh_events = unseen_events(self.store(), &events);

            self.inner
                .store
                .apply_integration_snapshot(crate::store::RefreshSnapshot {
                    devices,
                    clients,
                    networks: Vec::new(),
                    wifi: Vec::new(),
                    policies: Vec::new(),
                    zones: Vec::new(),
                    acls: Vec::new(),
                    dns: Vec::new(),
                    vouchers: Vec::new(),
                    sites,
                    events,
                    traffic_matching_lists: Vec::new(),
                });

            for event in fresh_events {
                let _ = self.inner.event_tx.send(Arc::new(event));
            }
        }

        debug!(
            devices = self.inner.store.device_count(),
            clients = self.inner.store.client_count(),
            "data refresh complete"
        );

        Ok(())
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

    // ── State observation ────────────────────────────────────────

    /// Subscribe to connection state changes.
    pub fn connection_state(&self) -> watch::Receiver<ConnectionState> {
        self.inner.connection_state.subscribe()
    }

    /// Subscribe to the event broadcast stream.
    pub fn events(&self) -> broadcast::Receiver<Arc<Event>> {
        self.inner.event_tx.subscribe()
    }

    // ── Snapshot accessors (delegate to DataStore) ───────────────

    pub fn devices_snapshot(&self) -> Arc<Vec<Arc<Device>>> {
        self.inner.store.devices_snapshot()
    }

    pub fn clients_snapshot(&self) -> Arc<Vec<Arc<Client>>> {
        self.inner.store.clients_snapshot()
    }

    pub fn networks_snapshot(&self) -> Arc<Vec<Arc<Network>>> {
        self.inner.store.networks_snapshot()
    }

    pub fn wifi_broadcasts_snapshot(&self) -> Arc<Vec<Arc<WifiBroadcast>>> {
        self.inner.store.wifi_broadcasts_snapshot()
    }

    pub fn firewall_policies_snapshot(&self) -> Arc<Vec<Arc<FirewallPolicy>>> {
        self.inner.store.firewall_policies_snapshot()
    }

    pub fn firewall_zones_snapshot(&self) -> Arc<Vec<Arc<FirewallZone>>> {
        self.inner.store.firewall_zones_snapshot()
    }

    pub fn acl_rules_snapshot(&self) -> Arc<Vec<Arc<AclRule>>> {
        self.inner.store.acl_rules_snapshot()
    }

    pub fn dns_policies_snapshot(&self) -> Arc<Vec<Arc<DnsPolicy>>> {
        self.inner.store.dns_policies_snapshot()
    }

    pub fn vouchers_snapshot(&self) -> Arc<Vec<Arc<Voucher>>> {
        self.inner.store.vouchers_snapshot()
    }

    pub fn sites_snapshot(&self) -> Arc<Vec<Arc<Site>>> {
        self.inner.store.sites_snapshot()
    }

    pub fn events_snapshot(&self) -> Arc<Vec<Arc<Event>>> {
        self.inner.store.events_snapshot()
    }

    pub fn traffic_matching_lists_snapshot(&self) -> Arc<Vec<Arc<TrafficMatchingList>>> {
        self.inner.store.traffic_matching_lists_snapshot()
    }

    // ── Stream accessors (delegate to DataStore) ─────────────────

    pub fn devices(&self) -> EntityStream<Device> {
        self.inner.store.subscribe_devices()
    }

    pub fn clients(&self) -> EntityStream<Client> {
        self.inner.store.subscribe_clients()
    }

    pub fn networks(&self) -> EntityStream<Network> {
        self.inner.store.subscribe_networks()
    }

    pub fn wifi_broadcasts(&self) -> EntityStream<WifiBroadcast> {
        self.inner.store.subscribe_wifi_broadcasts()
    }

    pub fn firewall_policies(&self) -> EntityStream<FirewallPolicy> {
        self.inner.store.subscribe_firewall_policies()
    }

    pub fn firewall_zones(&self) -> EntityStream<FirewallZone> {
        self.inner.store.subscribe_firewall_zones()
    }

    pub fn acl_rules(&self) -> EntityStream<AclRule> {
        self.inner.store.subscribe_acl_rules()
    }

    pub fn dns_policies(&self) -> EntityStream<DnsPolicy> {
        self.inner.store.subscribe_dns_policies()
    }

    pub fn vouchers(&self) -> EntityStream<Voucher> {
        self.inner.store.subscribe_vouchers()
    }

    pub fn sites(&self) -> EntityStream<Site> {
        self.inner.store.subscribe_sites()
    }

    pub fn traffic_matching_lists(&self) -> EntityStream<TrafficMatchingList> {
        self.inner.store.subscribe_traffic_matching_lists()
    }

    /// Subscribe to site health updates (WAN IP, latency, bandwidth rates).
    pub fn site_health(&self) -> watch::Receiver<Arc<Vec<HealthSummary>>> {
        self.inner.store.subscribe_site_health()
    }

    /// Drain warnings accumulated during connect (e.g. Legacy auth failure).
    pub async fn take_warnings(&self) -> Vec<String> {
        std::mem::take(&mut *self.inner.warnings.lock().await)
    }

    /// Whether a logged-in Legacy client is available for legacy-only features.
    pub async fn has_legacy_access(&self) -> bool {
        self.inner.legacy_client.lock().await.is_some()
    }

    /// Whether the Integration API is available for integration-backed features.
    pub async fn has_integration_access(&self) -> bool {
        self.inner.integration_client.lock().await.is_some()
            && self.inner.site_id.lock().await.is_some()
    }

    // ── Ad-hoc Integration API queries ───────────────────────────
    //
    // These bypass the DataStore and query the Integration API directly.
    // Intended for reference data that doesn't need reactive subscriptions.

    /// Fetch VPN servers from the Integration API.
    pub async fn list_vpn_servers(&self) -> Result<Vec<VpnServer>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_vpn_servers")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_vpn_servers(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|s| {
                let id = s
                    .fields
                    .get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                VpnServer {
                    id,
                    name: s
                        .fields
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    server_type: s
                        .fields
                        .get("type")
                        .or_else(|| s.fields.get("serverType"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_owned(),
                    enabled: s.fields.get("enabled").and_then(serde_json::Value::as_bool),
                }
            })
            .collect())
    }

    /// Fetch VPN tunnels from the Integration API.
    pub async fn list_vpn_tunnels(&self) -> Result<Vec<VpnTunnel>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_vpn_tunnels")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_vpn_tunnels(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|t| {
                let id = t
                    .fields
                    .get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                VpnTunnel {
                    id,
                    name: t
                        .fields
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    tunnel_type: t
                        .fields
                        .get("type")
                        .or_else(|| t.fields.get("tunnelType"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_owned(),
                    enabled: t.fields.get("enabled").and_then(serde_json::Value::as_bool),
                }
            })
            .collect())
    }

    /// Fetch WAN interfaces from the Integration API.
    pub async fn list_wans(&self) -> Result<Vec<WanInterface>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_wans")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_wans(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|w| {
                let id = w
                    .fields
                    .get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                let parse_ip = |key: &str| -> Option<std::net::IpAddr> {
                    w.fields
                        .get(key)
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse().ok())
                };
                let dns = w
                    .fields
                    .get("dns")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().and_then(|s| s.parse().ok()))
                            .collect()
                    })
                    .unwrap_or_default();
                WanInterface {
                    id,
                    name: w
                        .fields
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    ip: parse_ip("ipAddress").or_else(|| parse_ip("ip")),
                    gateway: parse_ip("gateway"),
                    dns,
                }
            })
            .collect())
    }

    /// Fetch DPI categories from the Integration API.
    pub async fn list_dpi_categories(&self) -> Result<Vec<DpiCategory>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_dpi_categories")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_dpi_categories(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|c| {
                #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                let id = c
                    .fields
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0) as u32;
                DpiCategory {
                    id,
                    name: c
                        .fields
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_owned(),
                    tx_bytes: c
                        .fields
                        .get("txBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                    rx_bytes: c
                        .fields
                        .get("rxBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                    apps: Vec::new(),
                }
            })
            .collect())
    }

    /// Fetch DPI applications from the Integration API.
    pub async fn list_dpi_applications(&self) -> Result<Vec<DpiApplication>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_dpi_applications")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_dpi_applications(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|a| {
                #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                let id = a
                    .fields
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0) as u32;
                DpiApplication {
                    id,
                    name: a
                        .fields
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_owned(),
                    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                    category_id: a
                        .fields
                        .get("categoryId")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as u32,
                    tx_bytes: a
                        .fields
                        .get("txBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                    rx_bytes: a
                        .fields
                        .get("rxBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                }
            })
            .collect())
    }

    /// Fetch RADIUS profiles from the Integration API.
    pub async fn list_radius_profiles(&self) -> Result<Vec<RadiusProfile>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_radius_profiles")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_radius_profiles(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|r| {
                let id = r
                    .fields
                    .get("id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| uuid::Uuid::parse_str(s).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                RadiusProfile {
                    id,
                    name: r
                        .fields
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_owned(),
                }
            })
            .collect())
    }

    /// Fetch countries from the Integration API.
    pub async fn list_countries(&self) -> Result<Vec<Country>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let ic = guard
            .as_ref()
            .ok_or_else(|| unsupported("list_countries"))?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_countries(off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|c| Country {
                code: c
                    .fields
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned(),
                name: c
                    .fields
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_owned(),
            })
            .collect())
    }

    /// Fetch references for a specific network (Integration API).
    pub async fn get_network_references(
        &self,
        network_id: &EntityId,
    ) -> Result<serde_json::Value, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "get_network_references")?;
        let uuid = require_uuid(network_id)?;
        let refs = ic.get_network_references(&sid, &uuid).await?;
        Ok(serde_json::to_value(refs).unwrap_or_default())
    }

    /// Fetch firewall policy ordering (Integration API).
    pub async fn get_firewall_policy_ordering(
        &self,
        source_zone_id: &EntityId,
        destination_zone_id: &EntityId,
    ) -> Result<crate::integration_types::FirewallPolicyOrdering, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) =
            require_integration(guard.as_ref(), site_id, "get_firewall_policy_ordering")?;
        let source_zone_uuid = require_uuid(source_zone_id)?;
        let destination_zone_uuid = require_uuid(destination_zone_id)?;
        Ok(ic
            .get_firewall_policy_ordering(&sid, &source_zone_uuid, &destination_zone_uuid)
            .await?)
    }

    /// Fetch ACL rule ordering (Integration API).
    pub async fn get_acl_rule_ordering(
        &self,
    ) -> Result<crate::integration_types::AclRuleOrdering, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "get_acl_rule_ordering")?;
        Ok(ic.get_acl_rule_ordering(&sid).await?)
    }

    /// List pending devices.
    ///
    /// Prefers Integration API pending endpoint, falls back to filtering
    /// the canonical device snapshot by pending adoption state.
    pub async fn list_pending_devices(&self) -> Result<Vec<serde_json::Value>, CoreError> {
        let integration_guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;

        if let (Some(ic), Some(sid)) = (integration_guard.as_ref(), site_id) {
            let raw = ic
                .paginate_all(200, |off, lim| ic.list_pending_devices(&sid, off, lim))
                .await?;
            return Ok(raw
                .into_iter()
                .map(|v| serde_json::to_value(v).unwrap_or_default())
                .collect());
        }

        let snapshot = self.devices_snapshot();
        Ok(snapshot
            .iter()
            .filter(|d| d.state == crate::model::DeviceState::PendingAdoption)
            .map(|d| serde_json::to_value(d.as_ref()).unwrap_or_default())
            .collect())
    }

    /// List device tags.
    ///
    /// Uses Integration API when available.
    pub async fn list_device_tags(&self) -> Result<Vec<serde_json::Value>, CoreError> {
        let integration_guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        if let (Some(ic), Some(sid)) = (integration_guard.as_ref(), site_id) {
            let raw = ic
                .paginate_all(200, |off, lim| ic.list_device_tags(&sid, off, lim))
                .await?;
            return Ok(raw
                .into_iter()
                .map(|v| serde_json::to_value(v).unwrap_or_default())
                .collect());
        }

        Ok(Vec::new())
    }

    /// List controller backups (legacy API).
    pub async fn list_backups(&self) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.list_backups().await?)
    }

    /// Download a controller backup file (legacy API).
    pub async fn download_backup(&self, filename: &str) -> Result<Vec<u8>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.download_backup(filename).await?)
    }

    // ── Statistics (Legacy API) ────────────────────────────────────

    /// Fetch site-level historical statistics.
    pub async fn get_site_stats(
        &self,
        interval: &str,
        start: Option<i64>,
        end: Option<i64>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_site_stats(interval, start, end, attrs).await?)
    }

    /// Fetch per-device historical statistics.
    pub async fn get_device_stats(
        &self,
        interval: &str,
        macs: Option<&[String]>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_device_stats(interval, macs, attrs).await?)
    }

    /// Fetch per-client historical statistics.
    pub async fn get_client_stats(
        &self,
        interval: &str,
        macs: Option<&[String]>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_client_stats(interval, macs, attrs).await?)
    }

    /// Fetch gateway historical statistics.
    pub async fn get_gateway_stats(
        &self,
        interval: &str,
        start: Option<i64>,
        end: Option<i64>,
        attrs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy
            .get_gateway_stats(interval, start, end, attrs)
            .await?)
    }

    /// Fetch DPI statistics.
    pub async fn get_dpi_stats(
        &self,
        group_by: &str,
        macs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_dpi_stats(group_by, macs).await?)
    }

    // ── Ad-hoc Legacy API queries ──────────────────────────────────
    //
    // Legacy-only data that doesn't live in the DataStore.

    /// Fetch admin list from the Legacy API.
    pub async fn list_admins(&self) -> Result<Vec<Admin>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.list_admins().await?;
        Ok(raw
            .into_iter()
            .map(|v| Admin {
                id: v.get("_id").and_then(|v| v.as_str()).map_or_else(
                    || EntityId::Legacy("unknown".into()),
                    |s| EntityId::Legacy(s.into()),
                ),
                name: v
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned(),
                email: v.get("email").and_then(|v| v.as_str()).map(String::from),
                role: v
                    .get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned(),
                is_super: v
                    .get("is_super")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                last_login: None,
            })
            .collect())
    }

    /// Fetch alarms from the Legacy API.
    pub async fn list_alarms(&self) -> Result<Vec<Alarm>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.list_alarms().await?;
        Ok(raw.into_iter().map(Alarm::from).collect())
    }

    /// Fetch controller system info.
    ///
    /// Prefers the Integration API (`GET /v1/info`) when available,
    /// falls back to Legacy `stat/sysinfo`.
    pub async fn get_system_info(&self) -> Result<SystemInfo, CoreError> {
        // Try Integration API first (works with API key auth).
        {
            let guard = self.inner.integration_client.lock().await;
            if let Some(ic) = guard.as_ref() {
                let info = ic.get_info().await?;
                let f = &info.fields;
                return Ok(SystemInfo {
                    controller_name: f
                        .get("applicationName")
                        .or_else(|| f.get("name"))
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    version: f
                        .get("applicationVersion")
                        .or_else(|| f.get("version"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_owned(),
                    build: f.get("build").and_then(|v| v.as_str()).map(String::from),
                    hostname: f.get("hostname").and_then(|v| v.as_str()).map(String::from),
                    ip: None, // Not available via Integration API
                    uptime_secs: f.get("uptime").and_then(serde_json::Value::as_u64),
                    update_available: f
                        .get("isUpdateAvailable")
                        .or_else(|| f.get("update_available"))
                        .and_then(serde_json::Value::as_bool),
                });
            }
        }

        // Fallback to Legacy API (requires session auth).
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_sysinfo().await?;
        Ok(SystemInfo {
            controller_name: raw
                .get("controller_name")
                .or_else(|| raw.get("name"))
                .and_then(|v| v.as_str())
                .map(String::from),
            version: raw
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            build: raw.get("build").and_then(|v| v.as_str()).map(String::from),
            hostname: raw
                .get("hostname")
                .and_then(|v| v.as_str())
                .map(String::from),
            ip: raw
                .get("ip_addrs")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()),
            uptime_secs: raw.get("uptime").and_then(serde_json::Value::as_u64),
            update_available: raw
                .get("update_available")
                .and_then(serde_json::Value::as_bool),
        })
    }

    /// Fetch site health dashboard from the Legacy API.
    pub async fn get_site_health(&self) -> Result<Vec<HealthSummary>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_health().await?;
        Ok(convert_health_summaries(raw))
    }

    /// Fetch low-level sysinfo from the Legacy API.
    pub async fn get_sysinfo(&self) -> Result<SysInfo, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_sysinfo().await?;
        Ok(SysInfo {
            timezone: raw
                .get("timezone")
                .and_then(|v| v.as_str())
                .map(String::from),
            autobackup: raw.get("autobackup").and_then(serde_json::Value::as_bool),
            hostname: raw
                .get("hostname")
                .and_then(|v| v.as_str())
                .map(String::from),
            ip_addrs: raw
                .get("ip_addrs")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            live_chat: raw
                .get("live_chat")
                .and_then(|v| v.as_str())
                .map(String::from),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            data_retention_days: raw
                .get("data_retention_days")
                .and_then(serde_json::Value::as_u64)
                .map(|n| n as u32),
            extra: raw,
        })
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

/// Periodically refresh data from the controller.
async fn refresh_task(controller: Controller, interval_secs: u64, cancel: CancellationToken) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    interval.tick().await; // consume the immediate first tick

    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => break,
            _ = interval.tick() => {
                if let Err(e) = controller.full_refresh().await {
                    warn!(error = %e, "periodic refresh failed");
                }
            }
        }
    }
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

/// Downgrade a paginated result to an empty `Vec` when the endpoint returns 404.
///
/// Some Integration API endpoints (ACL rules, DNS policies, vouchers) are not
/// available on all controller firmware versions. Rather than failing the entire
/// refresh, we log a debug message and return an empty collection.
fn unwrap_or_empty<S, D>(endpoint: &str, result: Result<Vec<S>, crate::error::Error>) -> Vec<D>
where
    D: From<S>,
{
    match result {
        Ok(items) => items.into_iter().map(D::from).collect(),
        Err(ref e) if e.is_not_found() => {
            debug!("{endpoint}: not available (404), treating as empty");
            Vec::new()
        }
        Err(e) => {
            warn!("{endpoint}: unexpected error {e}, treating as empty");
            Vec::new()
        }
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

fn unseen_events(store: &DataStore, events: &[Event]) -> Vec<Event> {
    let mut seen: HashSet<String> = store
        .events_snapshot()
        .iter()
        .map(|event| event_storage_key(event))
        .collect();

    events
        .iter()
        .filter(|event| seen.insert(event_storage_key(event)))
        .cloned()
        .collect()
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
