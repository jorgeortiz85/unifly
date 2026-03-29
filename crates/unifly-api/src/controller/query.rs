use super::*;

impl Controller {
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

    pub async fn list_vpn_servers(&self) -> Result<Vec<VpnServer>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_vpn_servers")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_vpn_servers(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|server| {
                let id = server
                    .fields
                    .get("id")
                    .and_then(|value| value.as_str())
                    .and_then(|value| uuid::Uuid::parse_str(value).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                VpnServer {
                    id,
                    name: server
                        .fields
                        .get("name")
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    server_type: server
                        .fields
                        .get("type")
                        .or_else(|| server.fields.get("serverType"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_owned(),
                    enabled: server
                        .fields
                        .get("enabled")
                        .and_then(serde_json::Value::as_bool),
                }
            })
            .collect())
    }

    pub async fn list_vpn_tunnels(&self) -> Result<Vec<VpnTunnel>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_vpn_tunnels")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_vpn_tunnels(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|tunnel| {
                let id = tunnel
                    .fields
                    .get("id")
                    .and_then(|value| value.as_str())
                    .and_then(|value| uuid::Uuid::parse_str(value).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                VpnTunnel {
                    id,
                    name: tunnel
                        .fields
                        .get("name")
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    tunnel_type: tunnel
                        .fields
                        .get("type")
                        .or_else(|| tunnel.fields.get("tunnelType"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("UNKNOWN")
                        .to_owned(),
                    enabled: tunnel
                        .fields
                        .get("enabled")
                        .and_then(serde_json::Value::as_bool),
                }
            })
            .collect())
    }

    pub async fn list_wans(&self) -> Result<Vec<WanInterface>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_wans")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_wans(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|wan| {
                let id = wan
                    .fields
                    .get("id")
                    .and_then(|value| value.as_str())
                    .and_then(|value| uuid::Uuid::parse_str(value).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                let parse_ip = |key: &str| -> Option<std::net::IpAddr> {
                    wan.fields
                        .get(key)
                        .and_then(|value| value.as_str())
                        .and_then(|value| value.parse().ok())
                };
                let dns = wan
                    .fields
                    .get("dns")
                    .and_then(|value| value.as_array())
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_str().and_then(|value| value.parse().ok()))
                            .collect()
                    })
                    .unwrap_or_default();
                WanInterface {
                    id,
                    name: wan
                        .fields
                        .get("name")
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    ip: parse_ip("ipAddress").or_else(|| parse_ip("ip")),
                    gateway: parse_ip("gateway"),
                    dns,
                }
            })
            .collect())
    }

    pub async fn list_dpi_categories(&self) -> Result<Vec<DpiCategory>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_dpi_categories")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_dpi_categories(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|category| {
                #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                let id = category
                    .fields
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0) as u32;
                DpiCategory {
                    id,
                    name: category
                        .fields
                        .get("name")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unknown")
                        .to_owned(),
                    tx_bytes: category
                        .fields
                        .get("txBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                    rx_bytes: category
                        .fields
                        .get("rxBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                    apps: Vec::new(),
                }
            })
            .collect())
    }

    pub async fn list_dpi_applications(&self) -> Result<Vec<DpiApplication>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_dpi_applications")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_dpi_applications(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|application| {
                #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                let id = application
                    .fields
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0) as u32;
                DpiApplication {
                    id,
                    name: application
                        .fields
                        .get("name")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unknown")
                        .to_owned(),
                    #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
                    category_id: application
                        .fields
                        .get("categoryId")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0) as u32,
                    tx_bytes: application
                        .fields
                        .get("txBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                    rx_bytes: application
                        .fields
                        .get("rxBytes")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or(0),
                }
            })
            .collect())
    }

    pub async fn list_radius_profiles(&self) -> Result<Vec<RadiusProfile>, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "list_radius_profiles")?;
        let raw = ic
            .paginate_all(200, |off, lim| ic.list_radius_profiles(&sid, off, lim))
            .await?;
        Ok(raw
            .into_iter()
            .map(|profile| {
                let id = profile
                    .fields
                    .get("id")
                    .and_then(|value| value.as_str())
                    .and_then(|value| uuid::Uuid::parse_str(value).ok())
                    .map_or_else(|| EntityId::Legacy("unknown".into()), EntityId::Uuid);
                RadiusProfile {
                    id,
                    name: profile
                        .fields
                        .get("name")
                        .and_then(|value| value.as_str())
                        .unwrap_or("Unknown")
                        .to_owned(),
                }
            })
            .collect())
    }

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
            .map(|country| Country {
                code: country
                    .fields
                    .get("code")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_owned(),
                name: country
                    .fields
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Unknown")
                    .to_owned(),
            })
            .collect())
    }

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

    pub async fn get_acl_rule_ordering(
        &self,
    ) -> Result<crate::integration_types::AclRuleOrdering, CoreError> {
        let guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        let (ic, sid) = require_integration(guard.as_ref(), site_id, "get_acl_rule_ordering")?;
        Ok(ic.get_acl_rule_ordering(&sid).await?)
    }

    pub async fn list_pending_devices(&self) -> Result<Vec<serde_json::Value>, CoreError> {
        let integration_guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;

        if let (Some(ic), Some(sid)) = (integration_guard.as_ref(), site_id) {
            let raw = ic
                .paginate_all(200, |off, lim| ic.list_pending_devices(&sid, off, lim))
                .await?;
            return Ok(raw
                .into_iter()
                .map(|value| serde_json::to_value(value).unwrap_or_default())
                .collect());
        }

        let snapshot = self.devices_snapshot();
        Ok(snapshot
            .iter()
            .filter(|device| device.state == crate::model::DeviceState::PendingAdoption)
            .map(|device| serde_json::to_value(device.as_ref()).unwrap_or_default())
            .collect())
    }

    pub async fn list_device_tags(&self) -> Result<Vec<serde_json::Value>, CoreError> {
        let integration_guard = self.inner.integration_client.lock().await;
        let site_id = *self.inner.site_id.lock().await;
        if let (Some(ic), Some(sid)) = (integration_guard.as_ref(), site_id) {
            let raw = ic
                .paginate_all(200, |off, lim| ic.list_device_tags(&sid, off, lim))
                .await?;
            return Ok(raw
                .into_iter()
                .map(|value| serde_json::to_value(value).unwrap_or_default())
                .collect());
        }

        Ok(Vec::new())
    }

    pub async fn list_backups(&self) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.list_backups().await?)
    }

    pub async fn download_backup(&self, filename: &str) -> Result<Vec<u8>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.download_backup(filename).await?)
    }

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

    pub async fn get_dpi_stats(
        &self,
        group_by: &str,
        macs: Option<&[String]>,
    ) -> Result<Vec<serde_json::Value>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        Ok(legacy.get_dpi_stats(group_by, macs).await?)
    }

    pub async fn list_admins(&self) -> Result<Vec<Admin>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.list_admins().await?;
        Ok(raw
            .into_iter()
            .map(|value| Admin {
                id: value
                    .get("_id")
                    .and_then(|value| value.as_str())
                    .map_or_else(
                        || EntityId::Legacy("unknown".into()),
                        |value| EntityId::Legacy(value.into()),
                    ),
                name: value
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .to_owned(),
                email: value
                    .get("email")
                    .and_then(|value| value.as_str())
                    .map(String::from),
                role: value
                    .get("role")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                    .to_owned(),
                is_super: value
                    .get("is_super")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                last_login: None,
            })
            .collect())
    }

    pub async fn list_alarms(&self) -> Result<Vec<Alarm>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.list_alarms().await?;
        Ok(raw.into_iter().map(Alarm::from).collect())
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo, CoreError> {
        {
            let guard = self.inner.integration_client.lock().await;
            if let Some(ic) = guard.as_ref() {
                let info = ic.get_info().await?;
                let fields = &info.fields;
                return Ok(SystemInfo {
                    controller_name: fields
                        .get("applicationName")
                        .or_else(|| fields.get("name"))
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    version: fields
                        .get("applicationVersion")
                        .or_else(|| fields.get("version"))
                        .and_then(|value| value.as_str())
                        .unwrap_or("unknown")
                        .to_owned(),
                    build: fields
                        .get("build")
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    hostname: fields
                        .get("hostname")
                        .and_then(|value| value.as_str())
                        .map(String::from),
                    ip: None,
                    uptime_secs: fields.get("uptime").and_then(serde_json::Value::as_u64),
                    update_available: fields
                        .get("isUpdateAvailable")
                        .or_else(|| fields.get("update_available"))
                        .and_then(serde_json::Value::as_bool),
                });
            }
        }

        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_sysinfo().await?;
        Ok(SystemInfo {
            controller_name: raw
                .get("controller_name")
                .or_else(|| raw.get("name"))
                .and_then(|value| value.as_str())
                .map(String::from),
            version: raw
                .get("version")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_owned(),
            build: raw
                .get("build")
                .and_then(|value| value.as_str())
                .map(String::from),
            hostname: raw
                .get("hostname")
                .and_then(|value| value.as_str())
                .map(String::from),
            ip: raw
                .get("ip_addrs")
                .and_then(|value| value.as_array())
                .and_then(|values| values.first())
                .and_then(|value| value.as_str())
                .and_then(|value| value.parse().ok()),
            uptime_secs: raw.get("uptime").and_then(serde_json::Value::as_u64),
            update_available: raw
                .get("update_available")
                .and_then(serde_json::Value::as_bool),
        })
    }

    pub async fn get_site_health(&self) -> Result<Vec<HealthSummary>, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_health().await?;
        Ok(convert_health_summaries(raw))
    }

    pub async fn get_sysinfo(&self) -> Result<SysInfo, CoreError> {
        let guard = self.inner.legacy_client.lock().await;
        let legacy = require_legacy(guard.as_ref())?;
        let raw = legacy.get_sysinfo().await?;
        Ok(SysInfo {
            timezone: raw
                .get("timezone")
                .and_then(|value| value.as_str())
                .map(String::from),
            autobackup: raw.get("autobackup").and_then(serde_json::Value::as_bool),
            hostname: raw
                .get("hostname")
                .and_then(|value| value.as_str())
                .map(String::from),
            ip_addrs: raw
                .get("ip_addrs")
                .and_then(|value| value.as_array())
                .map(|values| {
                    values
                        .iter()
                        .filter_map(|value| value.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            live_chat: raw
                .get("live_chat")
                .and_then(|value| value.as_str())
                .map(String::from),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            data_retention_days: raw
                .get("data_retention_days")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as u32),
            extra: raw,
        })
    }
}
