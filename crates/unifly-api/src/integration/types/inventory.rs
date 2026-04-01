use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Adopted device overview — from `GET /v1/sites/{siteId}/devices`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceResponse {
    pub id: Uuid,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub name: String,
    pub model: String,
    pub state: String,
    pub supported: bool,
    pub firmware_version: Option<String>,
    pub firmware_updatable: bool,
    pub features: Vec<String>,
    pub interfaces: Value,
}

/// Adopted device details — extends overview with additional fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDetailsResponse {
    pub id: Uuid,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub name: String,
    pub model: String,
    pub state: String,
    pub supported: bool,
    pub firmware_version: Option<String>,
    pub firmware_updatable: bool,
    pub features: Vec<String>,
    pub interfaces: Value,
    pub serial_number: Option<String>,
    pub short_name: Option<String>,
    pub startup_timestamp: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Latest statistics for a device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatisticsResponse {
    pub uptime_sec: Option<i64>,
    pub cpu_utilization_pct: Option<f64>,
    pub memory_utilization_pct: Option<f64>,
    pub load_average_1_min: Option<f64>,
    pub load_average_5_min: Option<f64>,
    pub load_average_15_min: Option<f64>,
    pub last_heartbeat_at: Option<String>,
    pub next_heartbeat_at: Option<String>,
    pub interfaces: Value,
    pub uplink: Option<Value>,
}

/// Client overview — from `GET /v1/sites/{siteId}/clients`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientResponse {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub client_type: String,
    pub ip_address: Option<String>,
    pub connected_at: Option<String>,
    pub mac_address: Option<String>,
    pub access: Value,
}

/// Client details — extends overview with additional fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientDetailsResponse {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "type")]
    pub client_type: String,
    pub ip_address: Option<String>,
    pub connected_at: Option<String>,
    pub mac_address: Option<String>,
    pub access: Value,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Device action request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceActionRequest {
    pub action: String,
}

/// Device adoption request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAdoptionRequest {
    pub mac_address: String,
    pub ignore_device_limit: bool,
}

/// Client action request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientActionRequest {
    pub action: String,
}

/// Client action response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientActionResponse {
    pub action: String,
    pub id: Uuid,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Port action request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortActionRequest {
    pub action: String,
}

/// Device tag — from `GET /v1/sites/{siteId}/devices/tags`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTagResponse {
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}

/// Pending device — from `GET /v1/sites/{siteId}/devices/pending`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingDeviceResponse {
    #[serde(flatten)]
    pub fields: HashMap<String, Value>,
}
