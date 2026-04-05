// ── Supporting / auxiliary domain types ──
//
// VPN, WAN, traffic matching, RADIUS, device tags, and other
// resources that don't warrant their own module.

use serde::{Deserialize, Serialize};
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
