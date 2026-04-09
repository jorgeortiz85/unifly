use std::collections::HashMap;

use serde_json::Value;

use crate::integration_types;
use crate::model::common::DataSource;
use crate::model::dns::{DnsPolicy, DnsPolicyType};
use crate::model::entity_id::EntityId;

use super::helpers::origin_from_metadata;

fn dns_value_from_extra(policy_type: DnsPolicyType, extra: &HashMap<String, Value>) -> String {
    match policy_type {
        DnsPolicyType::ARecord => extra
            .get("ipv4Address")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        DnsPolicyType::AaaaRecord => extra
            .get("ipv6Address")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        DnsPolicyType::CnameRecord => extra
            .get("targetDomain")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        DnsPolicyType::MxRecord => extra
            .get("mailServerDomain")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        DnsPolicyType::TxtRecord => extra
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        DnsPolicyType::SrvRecord => {
            let server = extra
                .get("serverDomain")
                .and_then(Value::as_str)
                .unwrap_or("");
            let service = extra.get("service").and_then(Value::as_str).unwrap_or("");
            let protocol = extra.get("protocol").and_then(Value::as_str).unwrap_or("");
            let port = extra.get("port").and_then(Value::as_u64);
            let priority = extra.get("priority").and_then(Value::as_u64);
            let weight = extra.get("weight").and_then(Value::as_u64);

            let mut parts = Vec::new();
            if !server.is_empty() {
                parts.push(server.to_owned());
            }
            if !service.is_empty() || !protocol.is_empty() {
                parts.push(format!("service={service}{protocol}"));
            }
            if let Some(port) = port {
                parts.push(format!("port={port}"));
            }
            if let Some(priority) = priority {
                parts.push(format!("priority={priority}"));
            }
            if let Some(weight) = weight {
                parts.push(format!("weight={weight}"));
            }
            parts.join(" ")
        }
        DnsPolicyType::ForwardDomain => extra
            .get("ipAddress")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    }
}

impl From<integration_types::DnsPolicyResponse> for DnsPolicy {
    fn from(d: integration_types::DnsPolicyResponse) -> Self {
        let policy_type = match d.policy_type.as_str() {
            "A" => DnsPolicyType::ARecord,
            "AAAA" => DnsPolicyType::AaaaRecord,
            "CNAME" => DnsPolicyType::CnameRecord,
            "MX" => DnsPolicyType::MxRecord,
            "TXT" => DnsPolicyType::TxtRecord,
            "SRV" => DnsPolicyType::SrvRecord,
            _ => DnsPolicyType::ForwardDomain,
        };

        DnsPolicy {
            id: EntityId::Uuid(d.id),
            policy_type,
            domain: d.domain.unwrap_or_default(),
            value: dns_value_from_extra(policy_type, &d.extra),
            #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
            ttl_seconds: d
                .extra
                .get("ttlSeconds")
                .and_then(serde_json::Value::as_u64)
                .map(|t| t as u32),
            origin: origin_from_metadata(&d.metadata),
            source: DataSource::IntegrationApi,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use serde_json::json;

    #[test]
    fn integration_dns_policy_uses_type_specific_fields() {
        let response = integration_types::DnsPolicyResponse {
            id: uuid::Uuid::nil(),
            policy_type: "A".into(),
            enabled: true,
            domain: Some("example.com".into()),
            metadata: json!({"origin": "USER"}),
            extra: HashMap::from([
                ("ipv4Address".into(), json!("192.168.1.10")),
                ("ttlSeconds".into(), json!(600)),
            ]),
        };

        let dns = DnsPolicy::from(response);
        assert_eq!(dns.value, "192.168.1.10");
        assert_eq!(dns.ttl_seconds, Some(600));
    }
}
