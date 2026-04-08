//! Data bridge — connects [Controller] streams to TUI actions.
//!
//! Runs as a background task: subscribes to entity streams and connection
//! state from the controller, forwarding every change as an [`Action`]
//! through the TUI's action channel.

use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};

use unifly_api::{ConnectionState, Controller, Event};

use crate::sanitizer::Sanitizer;
use crate::tui::action::Action;

/// Spawn the data bridge connecting [`Controller`] reactive streams to the TUI.
///
/// Connects to the controller, sends initial data snapshots, then loops
/// forwarding every entity change and connection-state transition as an
/// [`Action`]. Shuts down cleanly on cancellation.
///
/// When `sanitizer` is `Some`, all entity payloads are sanitized before
/// being sent to the TUI, replacing PII with deterministic fakes.
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub async fn spawn_data_bridge(
    controller: Controller,
    action_tx: mpsc::UnboundedSender<Action>,
    cancel: CancellationToken,
    sanitizer: Option<Arc<Sanitizer>>,
) {
    // Signal connecting state
    let _ = action_tx.send(Action::Reconnecting);

    if let Err(e) = controller.connect().await {
        warn!(error = %e, "failed to connect to controller");
        let _ = action_tx.send(Action::Disconnected(format!("{e}")));
        return;
    }

    let _ = action_tx.send(Action::Connected);

    // Surface any warnings from connect (e.g. Session auth failure)
    for warning in controller.take_warnings().await {
        let _ = action_tx.send(Action::Notify(crate::tui::action::Notification {
            message: warning,
            level: crate::tui::action::NotificationLevel::Warning,
        }));
    }

    // Subscribe to entity streams
    let mut devices = controller.devices();
    let mut clients = controller.clients();
    let mut networks = controller.networks();
    let mut fw_policies = controller.firewall_policies();
    let mut fw_zones = controller.firewall_zones();
    let mut acl_rules = controller.acl_rules();
    let mut nat_policies = controller.nat_policies();
    let mut wifi = controller.wifi_broadcasts();
    let mut events = controller.events();
    let mut conn_state = controller.connection_state();
    let mut site_health = controller.site_health();

    // Push initial snapshots so screens have data immediately.
    // When demo mode is active, every payload passes through the sanitizer.
    macro_rules! san_vec {
        ($snap:expr, $method:ident) => {
            match &sanitizer {
                Some(s) => s.$method(&$snap),
                None => $snap.clone(),
            }
        };
    }

    let _ = action_tx.send(Action::DevicesUpdated(san_vec!(
        devices.current(),
        sanitize_devices
    )));
    let _ = action_tx.send(Action::ClientsUpdated(san_vec!(
        clients.current(),
        sanitize_clients
    )));
    let _ = action_tx.send(Action::NetworksUpdated(san_vec!(
        networks.current(),
        sanitize_networks
    )));
    let _ = action_tx.send(Action::FirewallPoliciesUpdated(san_vec!(
        fw_policies.current(),
        sanitize_firewall_policies
    )));
    let _ = action_tx.send(Action::FirewallZonesUpdated(san_vec!(
        fw_zones.current(),
        sanitize_firewall_zones
    )));
    let _ = action_tx.send(Action::AclRulesUpdated(san_vec!(
        acl_rules.current(),
        sanitize_acl_rules
    )));
    let _ = action_tx.send(Action::NatPoliciesUpdated(san_vec!(
        nat_policies.current(),
        sanitize_nat_policies
    )));
    let _ = action_tx.send(Action::WifiBroadcastsUpdated(san_vec!(
        wifi.current(),
        sanitize_wifi_broadcasts
    )));

    // Push initial events from the DataStore snapshot (the broadcast channel
    // fires during connect(), before we subscribe, so those are lost).
    let events_snap: Vec<Arc<Event>> = controller.events_snapshot().to_vec();
    let events_snap = match &sanitizer {
        Some(s) => s.sanitize_events_vec(&events_snap),
        None => events_snap,
    };
    for evt in events_snap {
        let _ = action_tx.send(Action::EventReceived(evt));
    }

    // Push initial health snapshot
    let health_snap = site_health.borrow_and_update().clone();
    if !health_snap.is_empty() {
        let health_snap = match &sanitizer {
            Some(s) => s.sanitize_health_vec(&health_snap),
            None => health_snap,
        };
        let _ = action_tx.send(Action::HealthUpdated(health_snap));
    }

    // Stream loop — forward every change until cancelled
    loop {
        tokio::select! {
            biased;

            () = cancel.cancelled() => break,

            Some(d) = devices.changed() => {
                let d = san_vec!(d, sanitize_devices);
                let _ = action_tx.send(Action::DevicesUpdated(d));
            }
            Some(c) = clients.changed() => {
                let c = san_vec!(c, sanitize_clients);
                let _ = action_tx.send(Action::ClientsUpdated(c));
            }
            Some(n) = networks.changed() => {
                let n = san_vec!(n, sanitize_networks);
                let _ = action_tx.send(Action::NetworksUpdated(n));
            }
            Some(p) = fw_policies.changed() => {
                let p = san_vec!(p, sanitize_firewall_policies);
                let _ = action_tx.send(Action::FirewallPoliciesUpdated(p));
            }
            Some(z) = fw_zones.changed() => {
                let z = san_vec!(z, sanitize_firewall_zones);
                let _ = action_tx.send(Action::FirewallZonesUpdated(z));
            }
            Some(a) = acl_rules.changed() => {
                let a = san_vec!(a, sanitize_acl_rules);
                let _ = action_tx.send(Action::AclRulesUpdated(a));
            }
            Some(n) = nat_policies.changed() => {
                let n = san_vec!(n, sanitize_nat_policies);
                let _ = action_tx.send(Action::NatPoliciesUpdated(n));
            }
            Some(w) = wifi.changed() => {
                let w = san_vec!(w, sanitize_wifi_broadcasts);
                let _ = action_tx.send(Action::WifiBroadcastsUpdated(w));
            }
            Ok(event) = events.recv() => {
                let event = match &sanitizer {
                    Some(s) => Arc::new(s.sanitize_event(&event)),
                    None => event,
                };
                let _ = action_tx.send(Action::EventReceived(event));
            }
            Ok(()) = site_health.changed() => {
                let h = site_health.borrow_and_update().clone();
                let h = match &sanitizer {
                    Some(s) => s.sanitize_health_vec(&h),
                    None => h,
                };
                let _ = action_tx.send(Action::HealthUpdated(h));
            }
            Ok(()) = conn_state.changed() => {
                let state = conn_state.borrow_and_update().clone();
                match state {
                    ConnectionState::Connected => {
                        let _ = action_tx.send(Action::Connected);
                    }
                    ConnectionState::Disconnected => {
                        let _ = action_tx.send(Action::Disconnected("disconnected".into()));
                    }
                    ConnectionState::Reconnecting { .. } => {
                        let _ = action_tx.send(Action::Reconnecting);
                    }
                    ConnectionState::Failed => {
                        let _ = action_tx.send(Action::Disconnected("connection failed".into()));
                    }
                    ConnectionState::Connecting => {}
                }
            }
        }
    }

    controller.disconnect().await;
    debug!("data bridge shut down");
}
