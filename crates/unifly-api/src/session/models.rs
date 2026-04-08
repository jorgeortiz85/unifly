// Session API response types
//
// Models for the UniFi controller's session JSON API. All responses are wrapped
// in the `SessionResponse<T>` envelope. Fields use `#[serde(default)]` liberally
// because the API is inconsistent about field presence across firmware versions.

use serde::{Deserialize, Serialize};

// ── Response Envelope ────────────────────────────────────────────────

/// Standard UniFi session API response envelope.
///
/// Every session endpoint wraps its payload:
/// ```json
/// { "meta": { "rc": "ok", "msg": "optional" }, "data": [...] }
/// ```
#[derive(Debug, Deserialize)]
pub struct SessionResponse<T> {
    pub meta: Meta,
    pub data: Vec<T>,
}

/// Metadata from the session envelope. `rc` == `"ok"` means success.
#[derive(Debug, Deserialize)]
pub struct Meta {
    pub rc: String,
    #[serde(default)]
    pub msg: Option<String>,
}

// ── Device ───────────────────────────────────────────────────────────

/// Full device object from `stat/device`.
///
/// The session API can return 100+ fields per device. We model the most
/// commonly needed ones explicitly; everything else lands in `extra`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDevice {
    #[serde(default, rename = "_id")]
    pub id: String,
    pub mac: String,
    #[serde(rename = "type")]
    pub device_type: String,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub adopted: bool,
    /// 0=offline, 1=online, 2=pending, 4=upgrading, 5=provisioning
    #[serde(default)]
    pub state: i32,
    #[serde(default)]
    pub sys_stats: Option<SysStats>,
    #[serde(default)]
    pub uptime: Option<i64>,
    #[serde(default)]
    pub num_sta: Option<i32>,
    #[serde(default)]
    pub serial: Option<String>,
    #[serde(default)]
    pub site_id: Option<String>,
    #[serde(default)]
    pub last_seen: Option<i64>,
    #[serde(default)]
    pub upgradable: Option<bool>,
    #[serde(default, rename = "user-num_sta")]
    pub user_num_sta: Option<i32>,
    #[serde(default, rename = "guest-num_sta")]
    pub guest_num_sta: Option<i32>,
    /// Catch-all for undocumented fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// System statistics nested inside `SessionDevice`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysStats {
    #[serde(default, rename = "loadavg_1")]
    pub load_1: Option<String>,
    #[serde(default, rename = "loadavg_5")]
    pub load_5: Option<String>,
    #[serde(default, rename = "loadavg_15")]
    pub load_15: Option<String>,
    #[serde(default)]
    pub mem_total: Option<i64>,
    #[serde(default)]
    pub mem_used: Option<i64>,
    #[serde(default)]
    pub cpu: Option<String>,
}

// ── Client (Station) ─────────────────────────────────────────────────

/// Connected client from `stat/sta`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClientEntry {
    #[serde(rename = "_id")]
    pub id: String,
    pub mac: String,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub oui: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub is_guest: Option<bool>,
    #[serde(default)]
    pub is_wired: Option<bool>,
    #[serde(default)]
    pub authorized: Option<bool>,
    #[serde(default)]
    pub blocked: Option<bool>,
    #[serde(default)]
    pub signal: Option<i32>,
    #[serde(default)]
    pub tx_bytes: Option<i64>,
    #[serde(default)]
    pub rx_bytes: Option<i64>,
    #[serde(default)]
    pub tx_rate: Option<i64>,
    #[serde(default)]
    pub rx_rate: Option<i64>,
    #[serde(default)]
    pub uptime: Option<i64>,
    #[serde(default)]
    pub first_seen: Option<i64>,
    #[serde(default)]
    pub last_seen: Option<i64>,
    #[serde(default)]
    pub site_id: Option<String>,
    #[serde(default)]
    pub essid: Option<String>,
    #[serde(default)]
    pub bssid: Option<String>,
    #[serde(default)]
    pub channel: Option<i32>,
    #[serde(default)]
    pub radio: Option<String>,
    #[serde(default)]
    pub rssi: Option<i32>,
    #[serde(default)]
    pub noise: Option<i32>,
    #[serde(default)]
    pub satisfaction: Option<i32>,
    #[serde(default)]
    pub ap_mac: Option<String>,
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub network_id: Option<String>,
    #[serde(default)]
    pub sw_mac: Option<String>,
    #[serde(default)]
    pub sw_port: Option<i32>,
    /// Catch-all for undocumented fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ── User (known client / DHCP reservation) ──────────────────────────

/// User object from `rest/user`.
///
/// The "user" collection stores persistent client configuration such as
/// names, notes, and DHCP reservations. Unlike `stat/sta` (currently
/// connected stations), `rest/user` includes offline/historical clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUserEntry {
    #[serde(rename = "_id")]
    pub id: String,
    pub mac: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hostname: Option<String>,
    #[serde(default)]
    pub use_fixedip: Option<bool>,
    #[serde(default)]
    pub fixed_ip: Option<String>,
    #[serde(default)]
    pub network_id: Option<String>,
    #[serde(default)]
    pub site_id: Option<String>,
    #[serde(default)]
    pub noted: Option<bool>,
    #[serde(default)]
    pub note: Option<String>,
    /// Catch-all for undocumented fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ── Site ─────────────────────────────────────────────────────────────

/// Site object from `/api/self/sites`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSite {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub desc: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    /// Catch-all for undocumented fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ── Event ────────────────────────────────────────────────────────────

/// Event object from `stat/event`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub datetime: Option<String>,
    #[serde(default)]
    pub subsystem: Option<String>,
    #[serde(default)]
    pub site_id: Option<String>,
    /// Catch-all for undocumented fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ── Alarm ────────────────────────────────────────────────────────────

/// Alarm object from `stat/alarm`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAlarm {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub datetime: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    /// Catch-all for undocumented fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

// ── Wi-Fi Observability ─────────────────────────────────────────────

/// Neighboring / rogue access point from `stat/rogueap`.
///
/// Each entry represents a foreign AP detected by one of your APs.
/// Note: `stat/rogueap` uses Unix epoch **seconds** for query params,
/// unlike many other UniFi stats endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RogueAp {
    pub bssid: String,
    #[serde(default)]
    pub essid: Option<String>,
    #[serde(default)]
    pub channel: Option<i32>,
    #[serde(default)]
    pub freq: Option<i32>,
    #[serde(default)]
    pub signal: Option<i32>,
    #[serde(default)]
    pub rssi: Option<i32>,
    #[serde(default)]
    pub noise: Option<i32>,
    #[serde(default)]
    pub security: Option<String>,
    #[serde(default)]
    pub radio: Option<String>,
    #[serde(default)]
    pub age: Option<i64>,
    #[serde(default)]
    pub is_rogue: bool,
    /// MAC of your AP that observed this neighbor.
    #[serde(default)]
    pub ap_mac: Option<String>,
    /// Catch-all for undocumented fields.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// Country-level regulatory channel data from `stat/current-channel`.
///
/// The UniFi API returns one record per country with per-band channel lists
/// (e.g. `channels_ng`, `channels_na`, `channels_6e`) rather than per-radio
/// rows. The typed fields cover the most common bands; the `extra` map
/// captures width-specific and AFC lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAvailability {
    /// ISO 3166-1 numeric country code (e.g. `"840"` for the US).
    #[serde(default)]
    pub code: Option<String>,
    /// Two-letter country key (e.g. `"US"`).
    #[serde(default)]
    pub key: Option<String>,
    /// Human-readable country name.
    #[serde(default)]
    pub name: Option<String>,
    /// 2.4 GHz channels.
    #[serde(default)]
    pub channels_ng: Option<Vec<i32>>,
    /// 5 GHz channels.
    #[serde(default)]
    pub channels_na: Option<Vec<i32>>,
    /// 5 GHz DFS channels.
    #[serde(default)]
    pub channels_na_dfs: Option<Vec<i32>>,
    /// 6 GHz channels.
    #[serde(default)]
    pub channels_6e: Option<Vec<i32>>,
    /// Catch-all for width-specific lists, AFC data, etc.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
