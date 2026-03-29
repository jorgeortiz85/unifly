use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::{EntityId, FirewallAction};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrafficFilterSpec {
    Network {
        network_ids: Vec<String>,
        #[serde(default)]
        match_opposite: bool,
    },
    IpAddress {
        addresses: Vec<String>,
        #[serde(default)]
        match_opposite: bool,
    },
    Port {
        ports: Vec<String>,
        #[serde(default)]
        match_opposite: bool,
    },
}

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

#[cfg(test)]
mod tests {
    use super::{CreateAclRuleRequest, UpdateAclRuleRequest};
    use crate::model::FirewallAction;

    #[test]
    fn create_acl_rule_request_defaults_rule_type() {
        let request: CreateAclRuleRequest = serde_json::from_value(serde_json::json!({
            "name": "Allow IoT",
            "action": "Allow",
            "source_zone_id": "iot",
            "destination_zone_id": "lan",
            "enabled": true
        }))
        .unwrap();

        assert_eq!(request.rule_type, "IP");
    }

    #[test]
    fn update_acl_rule_request_serializes_type_field() {
        let request = UpdateAclRuleRequest {
            rule_type: Some("DEVICE".into()),
            action: Some(FirewallAction::Allow),
            ..Default::default()
        };

        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value.get("type").and_then(serde_json::Value::as_str),
            Some("DEVICE")
        );
        assert_eq!(value.get("rule_type"), None);
    }
}
