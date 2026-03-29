use super::*;

impl Controller {
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
}
