// ── Supporting / auxiliary domain types ──
//
// VPN, WAN, traffic matching, RADIUS, device tags, and other
// resources that don't warrant their own module.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::IpAddr;

use super::common::EntityOrigin;
use super::entity_id::EntityId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnServer {
    pub id: EntityId,
    pub name: Option<String>,
    pub server_type: String,
    pub enabled: Option<bool>,
    pub subnet: Option<String>,
    pub port: Option<u16>,
    pub wan_ip: Option<String>,
    pub connected_clients: Option<u32>,
    pub protocol: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnTunnel {
    pub id: EntityId,
    pub name: Option<String>,
    pub tunnel_type: String,
    pub enabled: Option<bool>,
    pub peer_address: Option<String>,
    pub local_subnets: Vec<String>,
    pub remote_subnets: Vec<String>,
    pub has_psk: bool,
    pub ike_version: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpsecSa {
    pub name: Option<String>,
    pub remote_ip: Option<String>,
    pub local_ip: Option<String>,
    pub state: Option<String>,
    pub tx_bytes: Option<i64>,
    pub rx_bytes: Option<i64>,
    pub uptime: Option<i64>,
    pub ike_version: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnSetting {
    pub key: String,
    pub enabled: Option<bool>,
    #[serde(default)]
    pub fields: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteToSiteVpn {
    pub id: EntityId,
    pub name: String,
    pub enabled: bool,
    pub vpn_type: String,
    pub remote_site_id: Option<String>,
    pub local_ip: Option<String>,
    pub interface: Option<String>,
    pub remote_host: Option<String>,
    #[serde(default)]
    pub remote_vpn_subnets: Vec<String>,
    #[serde(default)]
    pub fields: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteAccessVpnServer {
    pub id: EntityId,
    pub name: String,
    pub enabled: bool,
    pub vpn_type: String,
    pub local_port: Option<u16>,
    pub local_wan_ip: Option<String>,
    pub interface: Option<String>,
    pub gateway_subnet: Option<String>,
    pub radius_profile_id: Option<String>,
    pub exposed_to_site_vpn: Option<bool>,
    #[serde(default)]
    pub fields: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardPeer {
    pub id: EntityId,
    pub server_id: Option<EntityId>,
    pub name: String,
    pub interface_ip: Option<String>,
    pub interface_ipv6: Option<String>,
    pub public_key: Option<String>,
    #[serde(default)]
    pub allowed_ips: Vec<String>,
    pub has_private_key: bool,
    pub has_preshared_key: bool,
    #[serde(default)]
    pub fields: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnClientConnection {
    pub id: EntityId,
    pub name: Option<String>,
    pub connection_type: Option<String>,
    pub status: Option<String>,
    pub local_address: Option<String>,
    pub remote_address: Option<String>,
    pub username: Option<String>,
    #[serde(default)]
    pub fields: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnClientProfile {
    pub id: EntityId,
    pub name: String,
    pub enabled: bool,
    pub vpn_type: String,
    pub server_address: Option<String>,
    pub server_port: Option<u16>,
    pub local_address: Option<String>,
    pub username: Option<String>,
    pub interface: Option<String>,
    pub route_distance: Option<u32>,
    #[serde(default)]
    pub fields: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicSiteToSiteVpnConfig {
    pub id: EntityId,
    pub name: Option<String>,
    pub status: Option<String>,
    pub enabled: Option<bool>,
    pub local_site_name: Option<String>,
    pub remote_site_name: Option<String>,
    #[serde(default)]
    pub fields: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WanInterface {
    pub id: EntityId,
    pub name: Option<String>,
    pub ip: Option<IpAddr>,
    pub gateway: Option<IpAddr>,
    pub dns: Vec<IpAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficMatchingList {
    pub id: EntityId,
    pub name: String,
    /// List type: PORTS, IPV4_ADDRESSES, IPV6_ADDRESSES
    pub list_type: String,
    pub items: Vec<String>,
    pub origin: Option<EntityOrigin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusProfile {
    pub id: EntityId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTag {
    pub id: EntityId,
    pub name: String,
}
