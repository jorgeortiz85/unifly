use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{self, StreamExt};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use super::*;

impl Controller {
    /// Fetch all data from the controller and update the DataStore.
    ///
    /// Pulls devices, clients, and events from the controller APIs, converts
    /// them to domain types, and applies them to the store. Events are
    /// broadcast through the event channel after snapshot application.
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    pub async fn full_refresh(&self) -> Result<(), CoreError> {
        let integration = self.inner.integration_client.lock().await.clone();
        let site_id = *self.inner.site_id.lock().await;

        if let (Some(integration), Some(sid)) = (integration, site_id) {
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
                integration.paginate_all(50, |off, lim| integration.list_sites(off, lim)),
                integration.paginate_all(page_limit, |off, lim| {
                    integration.list_traffic_matching_lists(&sid, off, lim)
                }),
            );

            let devices: Vec<Device> = devices_res?.into_iter().map(Device::from).collect();
            let mut clients: Vec<Client> = clients_res?.into_iter().map(Client::from).collect();
            let network_ids: Vec<uuid::Uuid> = networks_res?
                .into_iter()
                .map(|network| network.id)
                .collect();
            info!(
                network_count = network_ids.len(),
                "fetching network details"
            );
            let networks: Vec<Network> = {
                stream::iter(network_ids.into_iter().map(|network_id| {
                    let integration = Arc::clone(&integration);
                    async move {
                        match integration.get_network(&sid, &network_id).await {
                            Ok(detail) => Some(Network::from(detail)),
                            Err(error) => {
                                warn!(network_id = %network_id, error = %error, "network detail fetch failed");
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

            let acls: Vec<AclRule> = unwrap_or_empty("acl/rules", acls_res);
            let dns: Vec<DnsPolicy> = unwrap_or_empty("dns/policies", dns_res);
            let vouchers: Vec<Voucher> = unwrap_or_empty("vouchers", vouchers_res);

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
                                Err(error) => {
                                    warn!(device = ?device.name, error = %error, "device stats fetch failed");
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

            #[allow(clippy::type_complexity)]
            let (legacy_events, legacy_health, legacy_clients, legacy_devices, legacy_users): (
                Vec<Event>,
                Vec<HealthSummary>,
                Vec<crate::legacy::models::LegacyClientEntry>,
                Vec<crate::legacy::models::LegacyDevice>,
                Vec<crate::legacy::models::LegacyUserEntry>,
            ) = match self.inner.legacy_client.lock().await.clone() {
                Some(legacy) => {
                    let (events_res, health_res, clients_res, devices_res, users_res) = tokio::join!(
                        legacy.list_events(Some(100)),
                        legacy.get_health(),
                        legacy.list_clients(),
                        legacy.list_devices(),
                        legacy.list_users(),
                    );

                    let events = match events_res {
                        Ok(raw) => raw.into_iter().map(Event::from).collect(),
                        Err(error) => {
                            warn!(error = %error, "legacy event fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    let health = match health_res {
                        Ok(raw) => convert_health_summaries(raw),
                        Err(error) => {
                            warn!(error = %error, "legacy health fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    let legacy_clients = match clients_res {
                        Ok(raw) => raw,
                        Err(error) => {
                            warn!(error = %error, "legacy client fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    let legacy_devices = match devices_res {
                        Ok(raw) => raw,
                        Err(error) => {
                            warn!(error = %error, "legacy device fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    let legacy_users = match users_res {
                        Ok(raw) => raw,
                        Err(error) => {
                            warn!(error = %error, "legacy user fetch failed (non-fatal)");
                            Vec::new()
                        }
                    };

                    (events, health, legacy_clients, legacy_devices, legacy_users)
                }
                None => (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            };

            if !legacy_clients.is_empty() {
                let legacy_by_ip: HashMap<&str, &crate::legacy::models::LegacyClientEntry> =
                    legacy_clients
                        .iter()
                        .filter_map(|client| client.ip.as_deref().map(|ip| (ip, client)))
                        .collect();
                let mut merged = 0u32;
                for client in &mut clients {
                    let ip_key = client.ip.map(|ip| ip.to_string());
                    if let Some(legacy_client) =
                        ip_key.as_deref().and_then(|ip| legacy_by_ip.get(ip))
                    {
                        if client.tx_bytes.is_none() {
                            client.tx_bytes = legacy_client
                                .tx_bytes
                                .and_then(|bytes| u64::try_from(bytes).ok());
                        }
                        if client.rx_bytes.is_none() {
                            client.rx_bytes = legacy_client
                                .rx_bytes
                                .and_then(|bytes| u64::try_from(bytes).ok());
                        }
                        if client.hostname.is_none() {
                            client.hostname.clone_from(&legacy_client.hostname);
                        }
                        if client.wireless.is_none() {
                            let legacy_client: Client = Client::from((*legacy_client).clone());
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

            if !legacy_users.is_empty() {
                let users_by_mac: HashMap<String, &crate::legacy::models::LegacyUserEntry> =
                    legacy_users
                        .iter()
                        .map(|user| (user.mac.to_lowercase(), user))
                        .collect();
                for client in &mut clients {
                    if let Some(user) = users_by_mac.get(&client.mac.as_str().to_lowercase()) {
                        client.use_fixedip = user.use_fixedip.unwrap_or(false);
                        client.fixed_ip = user.fixed_ip.as_deref().and_then(|ip| ip.parse().ok());
                    }
                }
            }

            if !legacy_devices.is_empty() {
                let legacy_by_mac: HashMap<&str, &crate::legacy::models::LegacyDevice> =
                    legacy_devices
                        .iter()
                        .map(|device| (device.mac.as_str(), device))
                        .collect();
                for device in &mut devices {
                    if let Some(legacy_device) = legacy_by_mac.get(device.mac.as_str()) {
                        if device.client_count.is_none() {
                            device.client_count = legacy_device
                                .num_sta
                                .and_then(|count| count.try_into().ok());
                        }
                        if device.wan_ipv6.is_none() {
                            device.wan_ipv6 = parse_legacy_device_wan_ipv6(&legacy_device.extra);
                        }
                    }
                }
            }

            if !legacy_health.is_empty() {
                self.inner
                    .store
                    .site_health
                    .send_modify(|health| *health = Arc::new(legacy_health));
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
}

/// Periodically refresh data from the controller.
pub(super) async fn refresh_task(
    controller: Controller,
    interval_secs: u64,
    cancel: CancellationToken,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    interval.tick().await;

    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => break,
            _ = interval.tick() => {
                if let Err(error) = controller.full_refresh().await {
                    warn!(error = %error, "periodic refresh failed");
                }
            }
        }
    }
}

/// Downgrade a paginated result to an empty `Vec` when the endpoint returns 404.
///
/// Some Integration API endpoints are optional on older controller firmware.
fn unwrap_or_empty<S, D>(endpoint: &str, result: Result<Vec<S>, crate::error::Error>) -> Vec<D>
where
    D: From<S>,
{
    match result {
        Ok(items) => items.into_iter().map(D::from).collect(),
        Err(ref error) if error.is_not_found() => {
            debug!("{endpoint}: not available (404), treating as empty");
            Vec::new()
        }
        Err(error) => {
            warn!("{endpoint}: unexpected error {error}, treating as empty");
            Vec::new()
        }
    }
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
