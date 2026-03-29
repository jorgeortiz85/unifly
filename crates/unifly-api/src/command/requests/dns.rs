use serde::{Deserialize, Serialize};

use crate::model::DnsPolicyType;

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

#[cfg(test)]
mod tests {
    use super::CreateDnsPolicyRequest;

    #[test]
    fn create_dns_policy_request_reads_ttl_alias() {
        let request: CreateDnsPolicyRequest = serde_json::from_value(serde_json::json!({
            "name": "Home DNS",
            "policy_type": "ARecord",
            "enabled": true,
            "domain": "printer.home",
            "value": "192.168.1.20",
            "ttlSeconds": 120
        }))
        .unwrap();

        assert_eq!(request.ttl_seconds, Some(120));
    }
}
