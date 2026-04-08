use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetPage<T> {
    pub data: Vec<T>,
    pub next_token: Option<String>,
    pub trace_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IspMetricInterval {
    FiveMinutes,
    OneHour,
}

impl IspMetricInterval {
    pub fn as_path_segment(self) -> &'static str {
        match self {
            Self::FiveMinutes => "5m",
            Self::OneHour => "1h",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "5m" => Some(Self::FiveMinutes),
            "1h" => Some(Self::OneHour),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    #[serde(default, alias = "hostId", alias = "_id")]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "firmwareVersion")]
    pub firmware_version: Option<String>,
    #[serde(default, rename = "macAddress")]
    pub mac_address: Option<String>,
    #[serde(default, rename = "reportedState")]
    pub reported_state: Option<Value>,
    #[serde(default, rename = "userData")]
    pub user_data: Option<Value>,
    #[serde(default, rename = "isOwner")]
    pub is_owner: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Host {
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| lookup_string(&self.extra, &["displayName", "consoleName", "name"]))
            .or_else(|| {
                self.user_data.as_ref().and_then(|value| {
                    lookup_nested_string(
                        value,
                        &[&["displayName"], &["consoleName"], &["name"], &["hostName"]],
                    )
                })
            })
            .or_else(|| {
                self.reported_state.as_ref().and_then(|value| {
                    lookup_nested_string(
                        value,
                        &[&["displayName"], &["consoleName"], &["name"], &["hostName"]],
                    )
                })
            })
            .unwrap_or_else(|| self.id.clone())
    }

    pub fn status(&self) -> String {
        lookup_string(&self.extra, &["status", "connectionState", "state"])
            .or_else(|| {
                self.reported_state.as_ref().and_then(|value| {
                    lookup_nested_string(value, &[&["status"], &["connectionState"], &["state"]])
                })
            })
            .or_else(|| {
                self.reported_state.as_ref().and_then(|value| {
                    lookup_nested_bool(value, &[&["isOnline"], &["online"], &["connected"]])
                        .map(bool_to_state)
                })
            })
            .unwrap_or_else(|| "unknown".into())
    }

    pub fn model_name(&self) -> String {
        self.model
            .clone()
            .or_else(|| lookup_string(&self.extra, &["hardwarePlatform", "productModel", "model"]))
            .or_else(|| {
                self.reported_state.as_ref().and_then(|value| {
                    lookup_nested_string(
                        value,
                        &[&["hardwarePlatform"], &["productModel"], &["model"]],
                    )
                })
            })
            .unwrap_or_default()
    }

    pub fn firmware(&self) -> String {
        self.firmware_version
            .clone()
            .or_else(|| lookup_string(&self.extra, &["firmwareVersion", "version"]))
            .or_else(|| {
                self.reported_state.as_ref().and_then(|value| {
                    lookup_nested_string(value, &[&["firmwareVersion"], &["version"]])
                })
            })
            .unwrap_or_default()
    }

    pub fn mac(&self) -> String {
        self.mac_address
            .clone()
            .or_else(|| lookup_string(&self.extra, &["macAddress", "mac"]))
            .unwrap_or_default()
    }

    pub fn is_owner_host(&self) -> bool {
        self.is_owner.unwrap_or_else(|| {
            lookup_bool(&self.extra, &["isOwner", "owner", "isOwnerHost"]).unwrap_or_else(|| {
                self.reported_state
                    .as_ref()
                    .and_then(|value| {
                        lookup_nested_bool(value, &[&["isOwner"], &["owner"], &["isOwnerHost"]])
                    })
                    .unwrap_or_else(|| {
                        lookup_string(&self.extra, &["hostType", "role"])
                            .or_else(|| {
                                self.reported_state.as_ref().and_then(|value| {
                                    lookup_nested_string(value, &[&["hostType"], &["role"]])
                                })
                            })
                            .is_some_and(|kind| kind.eq_ignore_ascii_case("owner"))
                    })
            })
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetSite {
    #[serde(default, alias = "siteId")]
    pub id: String,
    #[serde(default, rename = "hostId")]
    pub host_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub meta: Option<Value>,
    #[serde(default)]
    pub statistics: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl FleetSite {
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| lookup_string(&self.extra, &["displayName", "name"]))
            .or_else(|| {
                self.meta.as_ref().and_then(|value| {
                    lookup_nested_string(value, &[&["displayName"], &["name"], &["desc"]])
                })
            })
            .unwrap_or_else(|| self.id.clone())
    }

    pub fn device_count(&self) -> String {
        self.statistics
            .as_ref()
            .and_then(|value| {
                lookup_nested_u64(value, &[&["deviceCount"], &["devices"], &["numDevices"]])
            })
            .map(|count| count.to_string())
            .unwrap_or_default()
    }

    pub fn client_count(&self) -> String {
        self.statistics
            .as_ref()
            .and_then(|value| {
                lookup_nested_u64(value, &[&["clientCount"], &["clients"], &["numClients"]])
            })
            .map(|count| count.to_string())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudDevice {
    #[serde(default, alias = "deviceId", alias = "_id")]
    pub id: String,
    #[serde(default, rename = "hostId")]
    pub host_id: Option<String>,
    #[serde(default, rename = "siteId")]
    pub site_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "macAddress")]
    pub mac_address: Option<String>,
    #[serde(default, rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub uidb: Option<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl CloudDevice {
    pub fn display_name(&self) -> String {
        self.display_name
            .clone()
            .or_else(|| self.name.clone())
            .or_else(|| lookup_string(&self.extra, &["displayName", "name"]))
            .or_else(|| {
                self.uidb
                    .as_ref()
                    .and_then(|value| lookup_nested_string(value, &[&["displayName"], &["name"]]))
            })
            .unwrap_or_else(|| self.id.clone())
    }

    pub fn status(&self) -> String {
        lookup_string(&self.extra, &["status", "state", "connectionState"])
            .or_else(|| {
                self.uidb.as_ref().and_then(|value| {
                    lookup_nested_string(value, &[&["status"], &["state"], &["connectionState"]])
                })
            })
            .unwrap_or_else(|| "unknown".into())
    }

    pub fn model_name(&self) -> String {
        self.model
            .clone()
            .or_else(|| lookup_string(&self.extra, &["productModel", "displayModel", "model"]))
            .or_else(|| {
                self.uidb.as_ref().and_then(|value| {
                    lookup_nested_string(value, &[&["productModel"], &["displayModel"], &["model"]])
                })
            })
            .unwrap_or_default()
    }

    pub fn mac(&self) -> String {
        self.mac_address
            .clone()
            .or_else(|| lookup_string(&self.extra, &["macAddress", "mac"]))
            .or_else(|| {
                self.uidb
                    .as_ref()
                    .and_then(|value| lookup_nested_string(value, &[&["macAddress"], &["mac"]]))
            })
            .unwrap_or_default()
    }

    pub fn ip(&self) -> String {
        self.ip_address
            .clone()
            .or_else(|| lookup_string(&self.extra, &["ipAddress", "ip"]))
            .or_else(|| {
                self.uidb
                    .as_ref()
                    .and_then(|value| lookup_nested_string(value, &[&["ipAddress"], &["ip"]]))
            })
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IspMetric {
    #[serde(default, rename = "siteId")]
    pub site_id: Option<String>,
    #[serde(default, rename = "hostId")]
    pub host_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, rename = "downloadMbps")]
    pub download_mbps: Option<f64>,
    #[serde(default, rename = "uploadMbps")]
    pub upload_mbps: Option<f64>,
    #[serde(default, rename = "latencyMs")]
    pub latency_ms: Option<f64>,
    #[serde(default, rename = "packetLossPct")]
    pub packet_loss_pct: Option<f64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl IspMetric {
    pub fn status_text(&self) -> String {
        self.status
            .clone()
            .or_else(|| lookup_string(&self.extra, &["status"]))
            .unwrap_or_default()
    }

    pub fn timestamp_text(&self) -> String {
        self.timestamp
            .clone()
            .or_else(|| lookup_string(&self.extra, &["time", "timestamp"]))
            .unwrap_or_default()
    }

    pub fn latency_text(&self) -> String {
        self.latency_ms
            .or_else(|| lookup_f64(&self.extra, &["latencyMs", "latency"]))
            .map(|value| format!("{value:.1}"))
            .unwrap_or_default()
    }

    pub fn download_text(&self) -> String {
        self.download_mbps
            .or_else(|| lookup_f64(&self.extra, &["downloadMbps", "download"]))
            .map(|value| format!("{value:.1}"))
            .unwrap_or_default()
    }

    pub fn upload_text(&self) -> String {
        self.upload_mbps
            .or_else(|| lookup_f64(&self.extra, &["uploadMbps", "upload"]))
            .map(|value| format!("{value:.1}"))
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdWanConfig {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub hubs: Option<Vec<Value>>,
    #[serde(default)]
    pub sites: Option<Vec<Value>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl SdWanConfig {
    pub fn display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| lookup_string(&self.extra, &["displayName", "name"]))
            .unwrap_or_else(|| self.id.clone())
    }

    pub fn status_text(&self) -> String {
        self.status
            .clone()
            .or_else(|| lookup_string(&self.extra, &["deploymentStatus", "status"]))
            .unwrap_or_default()
    }

    pub fn hub_count(&self) -> String {
        self.hubs
            .as_ref()
            .map(std::vec::Vec::len)
            .or_else(|| lookup_array_len(&self.extra, &["hubs"]))
            .map(|count| count.to_string())
            .unwrap_or_default()
    }

    pub fn site_count(&self) -> String {
        self.sites
            .as_ref()
            .map(std::vec::Vec::len)
            .or_else(|| lookup_array_len(&self.extra, &["sites"]))
            .map(|count| count.to_string())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdWanStatus {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub progress: Option<f64>,
    #[serde(default)]
    pub errors: Option<Vec<Value>>,
    #[serde(default)]
    pub hubs: Option<Vec<Value>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl SdWanStatus {
    pub fn status_text(&self) -> String {
        self.status
            .clone()
            .or_else(|| lookup_string(&self.extra, &["deploymentStatus", "status"]))
            .unwrap_or_default()
    }

    pub fn progress_text(&self) -> String {
        self.progress
            .or_else(|| lookup_f64(&self.extra, &["progress"]))
            .map(|value| format!("{value:.0}%"))
            .unwrap_or_default()
    }

    pub fn error_count(&self) -> String {
        self.errors
            .as_ref()
            .map(std::vec::Vec::len)
            .or_else(|| lookup_array_len(&self.extra, &["errors"]))
            .map(|count| count.to_string())
            .unwrap_or_default()
    }
}

fn bool_to_state(value: bool) -> String {
    if value {
        "online".into()
    } else {
        "offline".into()
    }
}

fn lookup_string(extra: &BTreeMap<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| extra.get(*key).and_then(value_to_string))
}

fn lookup_bool(extra: &BTreeMap<String, Value>, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| extra.get(*key).and_then(value_to_bool))
}

fn lookup_f64(extra: &BTreeMap<String, Value>, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| extra.get(*key).and_then(value_to_f64))
}

fn lookup_array_len(extra: &BTreeMap<String, Value>, keys: &[&str]) -> Option<usize> {
    keys.iter().find_map(|key| {
        extra
            .get(*key)
            .and_then(|value| value.as_array().map(std::vec::Vec::len))
    })
}

fn lookup_nested_string(value: &Value, paths: &[&[&str]]) -> Option<String> {
    paths
        .iter()
        .find_map(|path| value_at_path(value, path).and_then(value_to_string))
}

fn lookup_nested_bool(value: &Value, paths: &[&[&str]]) -> Option<bool> {
    paths
        .iter()
        .find_map(|path| value_at_path(value, path).and_then(value_to_bool))
}

fn lookup_nested_u64(value: &Value, paths: &[&[&str]]) -> Option<u64> {
    paths
        .iter()
        .find_map(|path| value_at_path(value, path).and_then(value_to_u64))
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    Some(cursor)
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(text) if !text.trim().is_empty() => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn value_to_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        Value::String(text) => match text.to_ascii_lowercase().as_str() {
            "true" | "online" | "connected" => Some(true),
            "false" | "offline" | "disconnected" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn value_to_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}
