use serde::{Deserialize, Serialize};

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
