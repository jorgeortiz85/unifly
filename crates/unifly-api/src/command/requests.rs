// ── Typed request structs for Command payloads ──
//
// Every Command variant that previously took `serde_json::Value`
// now uses one of these strongly-typed request structs instead.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::{
    DnsPolicyType, EntityId, FirewallAction, NetworkManagement, NetworkPurpose, WifiSecurityMode,
};

// ── Network ────────────────────────────────────────────────────────

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

// ── WiFi ───────────────────────────────────────────────────────────

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

// ── Firewall Policy ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFirewallPolicyRequest {
    pub name: String,
    pub action: FirewallAction,
    pub source_zone_id: EntityId,
    pub destination_zone_id: EntityId,
    pub enabled: bool,
    pub logging_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_states: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_filter: Option<TrafficFilterSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_filter: Option<TrafficFilterSpec>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateFirewallPolicyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<FirewallAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_states: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_filter: Option<TrafficFilterSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_filter: Option<TrafficFilterSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging_enabled: Option<bool>,
}

/// Specification for building a traffic filter (used in create/update commands).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrafficFilterSpec {
    /// Filter by network IDs.
    Network {
        network_ids: Vec<String>,
        #[serde(default)]
        match_opposite: bool,
    },
    /// Filter by IP addresses (supports IPs, CIDRs, and ranges).
    IpAddress {
        addresses: Vec<String>,
        #[serde(default)]
        match_opposite: bool,
    },
    /// Filter by ports (supports single ports and ranges like "8000-9000").
    Port {
        ports: Vec<String>,
        #[serde(default)]
        match_opposite: bool,
    },
}

// ── Firewall Zone ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFirewallZoneRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub network_ids: Vec<EntityId>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateFirewallZoneRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_ids: Option<Vec<EntityId>>,
}

// ── ACL Rule ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAclRuleRequest {
    pub name: String,
    #[serde(default = "default_acl_rule_type")]
    pub rule_type: String,
    pub action: FirewallAction,
    pub source_zone_id: EntityId,
    pub destination_zone_id: EntityId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_filter: Option<TrafficFilterSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_filter: Option<TrafficFilterSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforcing_device_filter: Option<Value>,
    pub enabled: bool,
}

fn default_acl_rule_type() -> String {
    "IP".into()
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateAclRuleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub rule_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<FirewallAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_zone_id: Option<EntityId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_zone_id: Option<EntityId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_filter: Option<TrafficFilterSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_filter: Option<TrafficFilterSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforcing_device_filter: Option<Value>,
}

// ── DNS Policy ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDnsPolicyRequest {
    pub name: String,
    pub policy_type: DnsPolicyType,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ttlSeconds")]
    pub ttl_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ipv4Address")]
    pub ipv4_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ipv6Address")]
    pub ipv6_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "targetDomain")]
    pub target_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "mailServerDomain")]
    pub mail_server_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "serverDomain")]
    pub server_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateDnsPolicyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ttlSeconds")]
    pub ttl_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ipv4Address")]
    pub ipv4_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ipv6Address")]
    pub ipv6_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "targetDomain")]
    pub target_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "mailServerDomain")]
    pub mail_server_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "serverDomain")]
    pub server_domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u16>,
}

// ── Traffic Matching List ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTrafficMatchingListRequest {
    pub name: String,
    #[serde(default = "default_traffic_list_type")]
    pub list_type: String,
    pub entries: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "items")]
    pub raw_items: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTrafficMatchingListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "items")]
    pub raw_items: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_traffic_list_type() -> String {
    "IPV4".into()
}

// ── Vouchers ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVouchersRequest {
    pub count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_limit_minutes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_usage_limit_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_rate_limit_kbps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_rate_limit_kbps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_guest_limit: Option<u32>,
}
