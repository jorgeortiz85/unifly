use serde::{Deserialize, Serialize};

use crate::model::{EntityId, NetworkManagement, NetworkPurpose, WifiSecurityMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct CreateNetworkRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub management: Option<NetworkManagement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<NetworkPurpose>,
    pub dhcp_enabled: bool,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp_range_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp_range_stop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp_lease_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firewall_zone_id: Option<String>,
    pub isolation_enabled: bool,
    pub internet_access_enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct UpdateNetworkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_id: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dhcp_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolation_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internet_access_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdns_forwarding_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct CreateWifiBroadcastRequest {
    pub name: String,
    pub ssid: String,
    pub security_mode: WifiSecurityMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<EntityId>,
    #[serde(alias = "hideName")]
    pub hide_ssid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "broadcastingFrequenciesGHz")]
    pub frequencies_ghz: Option<Vec<f32>>,
    #[serde(default)]
    #[serde(alias = "bandSteeringEnabled")]
    pub band_steering: bool,
    #[serde(default)]
    #[serde(alias = "bssTransitionEnabled")]
    pub fast_roaming: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateWifiBroadcastRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_mode: Option<WifiSecurityMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "hideName")]
    pub hide_ssid: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::CreateWifiBroadcastRequest;

    #[test]
    fn create_wifi_broadcast_request_honors_alias_fields() {
        let request: CreateWifiBroadcastRequest = serde_json::from_value(serde_json::json!({
            "name": "Guest",
            "ssid": "Guest",
            "security_mode": "Open",
            "enabled": true,
            "hideName": true,
            "bandSteeringEnabled": true,
            "bssTransitionEnabled": true
        }))
        .unwrap();

        assert!(request.hide_ssid);
        assert!(request.band_steering);
        assert!(request.fast_roaming);
    }
}
