use crate::command::{Command, CommandResult};
use crate::core_error::CoreError;
use crate::model::FirewallAction;

use super::{
    CommandContext, build_acl_filter_value, build_create_dns_policy_fields, build_endpoint_json,
    build_update_dns_policy_fields, dns_policy_type_name, merge_acl_filter_value,
    require_integration, require_uuid, traffic_matching_list_items,
};

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub(super) async fn route(ctx: &CommandContext, cmd: Command) -> Result<CommandResult, CoreError> {
    let integration = ctx.integration.as_ref();
    let site_id = ctx.site_id;

    match cmd {
        Command::CreateFirewallPolicy(req) => {
            let (ic, sid) = require_integration(integration, site_id, "CreateFirewallPolicy")?;
            let action_str = match req.action {
                FirewallAction::Allow => "ALLOW",
                FirewallAction::Block => "DROP",
                FirewallAction::Reject => "REJECT",
            };
            let source =
                build_endpoint_json(&req.source_zone_id.to_string(), req.source_filter.as_ref());
            let destination = build_endpoint_json(
                &req.destination_zone_id.to_string(),
                req.destination_filter.as_ref(),
            );
            let ip_version = req.ip_version.as_deref().unwrap_or("IPV4_AND_IPV6");
            let body = crate::integration_types::FirewallPolicyCreateUpdate {
                name: req.name,
                description: req.description,
                enabled: req.enabled,
                action: serde_json::json!({ "type": action_str }),
                source,
                destination,
                ip_protocol_scope: serde_json::json!({ "ipVersion": ip_version }),
                logging_enabled: req.logging_enabled,
                ipsec_filter: None,
                schedule: None,
                connection_state_filter: req.connection_states,
            };
            ic.create_firewall_policy(&sid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::UpdateFirewallPolicy { id, update } => {
            let (ic, sid) = require_integration(integration, site_id, "UpdateFirewallPolicy")?;
            let uuid = require_uuid(&id)?;
            let existing = ic.get_firewall_policy(&sid, &uuid).await?;

            let source = if let Some(ref spec) = update.source_filter {
                let zone_id = existing
                    .source
                    .as_ref()
                    .and_then(|s| s.zone_id)
                    .map(|u| u.to_string())
                    .unwrap_or_default();
                build_endpoint_json(&zone_id, Some(spec))
            } else {
                serde_json::to_value(&existing.source).unwrap_or_default()
            };

            let destination = if let Some(ref spec) = update.destination_filter {
                let zone_id = existing
                    .destination
                    .as_ref()
                    .and_then(|d| d.zone_id)
                    .map(|u| u.to_string())
                    .unwrap_or_default();
                build_endpoint_json(&zone_id, Some(spec))
            } else {
                serde_json::to_value(&existing.destination).unwrap_or_default()
            };

            let action = if let Some(action) = update.action {
                let action_type = match action {
                    FirewallAction::Allow => "ALLOW",
                    FirewallAction::Block => "DROP",
                    FirewallAction::Reject => "REJECT",
                };
                serde_json::json!({ "type": action_type })
            } else {
                existing.action
            };

            let ip_protocol_scope = if let Some(ref version) = update.ip_version {
                serde_json::json!({ "ipVersion": version })
            } else {
                existing
                    .ip_protocol_scope
                    .unwrap_or_else(|| serde_json::json!({ "ipVersion": "IPV4_AND_IPV6" }))
            };

            let connection_state_filter = update.connection_states.or_else(|| {
                existing
                    .extra
                    .get("connectionStateFilter")
                    .and_then(serde_json::Value::as_array)
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(ToOwned::to_owned))
                            .collect::<Vec<_>>()
                    })
            });

            let payload = crate::integration_types::FirewallPolicyCreateUpdate {
                name: update.name.unwrap_or(existing.name),
                description: update.description.or(existing.description),
                enabled: update.enabled.unwrap_or(existing.enabled),
                action,
                source,
                destination,
                ip_protocol_scope,
                logging_enabled: update.logging_enabled.unwrap_or(existing.logging_enabled),
                ipsec_filter: existing
                    .extra
                    .get("ipsecFilter")
                    .and_then(serde_json::Value::as_str)
                    .map(ToOwned::to_owned),
                schedule: existing.extra.get("schedule").cloned(),
                connection_state_filter,
            };

            ic.update_firewall_policy(&sid, &uuid, &payload).await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteFirewallPolicy { id } => {
            let (ic, sid) = require_integration(integration, site_id, "DeleteFirewallPolicy")?;
            let uuid = require_uuid(&id)?;
            ic.delete_firewall_policy(&sid, &uuid).await?;
            Ok(CommandResult::Ok)
        }
        Command::PatchFirewallPolicy {
            id,
            enabled,
            logging,
        } => {
            let (ic, sid) = require_integration(integration, site_id, "PatchFirewallPolicy")?;
            let uuid = require_uuid(&id)?;
            let body = crate::integration_types::FirewallPolicyPatch {
                enabled,
                logging_enabled: logging,
            };
            ic.patch_firewall_policy(&sid, &uuid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::ReorderFirewallPolicies {
            zone_pair,
            ordered_ids,
        } => {
            let (ic, sid) = require_integration(integration, site_id, "ReorderFirewallPolicies")?;
            let source_zone_uuid = require_uuid(&zone_pair.0)?;
            let destination_zone_uuid = require_uuid(&zone_pair.1)?;
            let uuids: Result<Vec<uuid::Uuid>, _> = ordered_ids.iter().map(require_uuid).collect();
            let body = crate::integration_types::FirewallPolicyOrdering {
                before_system_defined: uuids?,
                after_system_defined: Vec::new(),
            };
            ic.set_firewall_policy_ordering(&sid, &source_zone_uuid, &destination_zone_uuid, &body)
                .await?;
            Ok(CommandResult::Ok)
        }
        Command::CreateFirewallZone(req) => {
            let (ic, sid) = require_integration(integration, site_id, "CreateFirewallZone")?;
            let network_uuids: Result<Vec<uuid::Uuid>, _> =
                req.network_ids.iter().map(require_uuid).collect();
            let body = crate::integration_types::FirewallZoneCreateUpdate {
                name: req.name,
                network_ids: network_uuids?,
            };
            ic.create_firewall_zone(&sid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::UpdateFirewallZone { id, update } => {
            let (ic, sid) = require_integration(integration, site_id, "UpdateFirewallZone")?;
            let uuid = require_uuid(&id)?;
            let existing = ic.get_firewall_zone(&sid, &uuid).await?;
            let network_ids = if let Some(ids) = update.network_ids {
                let uuids: Result<Vec<uuid::Uuid>, _> = ids.iter().map(require_uuid).collect();
                uuids?
            } else {
                existing.network_ids
            };
            let body = crate::integration_types::FirewallZoneCreateUpdate {
                name: update.name.unwrap_or(existing.name),
                network_ids,
            };
            ic.update_firewall_zone(&sid, &uuid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteFirewallZone { id } => {
            let (ic, sid) = require_integration(integration, site_id, "DeleteFirewallZone")?;
            let uuid = require_uuid(&id)?;
            ic.delete_firewall_zone(&sid, &uuid).await?;
            Ok(CommandResult::Ok)
        }
        Command::CreateAclRule(req) => {
            let (ic, sid) = require_integration(integration, site_id, "CreateAclRule")?;
            let action_str = match req.action {
                FirewallAction::Allow => "ALLOW",
                FirewallAction::Block => "BLOCK",
                FirewallAction::Reject => "REJECT",
            };
            let body = crate::integration_types::AclRuleCreateUpdate {
                name: req.name,
                rule_type: req.rule_type,
                action: action_str.into(),
                enabled: req.enabled,
                description: req.description,
                source_filter: Some(build_acl_filter_value(
                    &req.source_zone_id,
                    req.source_port.as_deref(),
                    req.protocol.as_deref(),
                )),
                destination_filter: Some(build_acl_filter_value(
                    &req.destination_zone_id,
                    req.destination_port.as_deref(),
                    req.protocol.as_deref(),
                )),
                enforcing_device_filter: req.enforcing_device_filter,
            };
            ic.create_acl_rule(&sid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::UpdateAclRule { id, update } => {
            let (ic, sid) = require_integration(integration, site_id, "UpdateAclRule")?;
            let uuid = require_uuid(&id)?;
            let existing = ic.get_acl_rule(&sid, &uuid).await?;
            let action_str = match update.action {
                Some(FirewallAction::Allow) => "ALLOW".into(),
                Some(FirewallAction::Block) => "BLOCK".into(),
                Some(FirewallAction::Reject) => "REJECT".into(),
                None => existing.action,
            };
            let body = crate::integration_types::AclRuleCreateUpdate {
                name: update.name.unwrap_or(existing.name),
                rule_type: update.rule_type.unwrap_or(existing.rule_type),
                action: action_str,
                enabled: update.enabled.unwrap_or(existing.enabled),
                description: update.description.or(existing.description),
                source_filter: merge_acl_filter_value(
                    existing.source_filter,
                    update.source_zone_id.as_ref(),
                    update.source_port.as_deref(),
                    update.protocol.as_deref(),
                ),
                destination_filter: merge_acl_filter_value(
                    existing.destination_filter,
                    update.destination_zone_id.as_ref(),
                    update.destination_port.as_deref(),
                    update.protocol.as_deref(),
                ),
                enforcing_device_filter: existing.enforcing_device_filter,
            };
            ic.update_acl_rule(&sid, &uuid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteAclRule { id } => {
            let (ic, sid) = require_integration(integration, site_id, "DeleteAclRule")?;
            let uuid = require_uuid(&id)?;
            ic.delete_acl_rule(&sid, &uuid).await?;
            Ok(CommandResult::Ok)
        }
        Command::ReorderAclRules { ordered_ids } => {
            let (ic, sid) = require_integration(integration, site_id, "ReorderAclRules")?;
            let uuids: Result<Vec<uuid::Uuid>, _> = ordered_ids.iter().map(require_uuid).collect();
            let body = crate::integration_types::AclRuleOrdering {
                ordered_acl_rule_ids: uuids?,
            };
            ic.set_acl_rule_ordering(&sid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::CreateDnsPolicy(req) => {
            let (ic, sid) = require_integration(integration, site_id, "CreateDnsPolicy")?;
            let policy_type_str = dns_policy_type_name(req.policy_type);
            let fields = build_create_dns_policy_fields(&req)?;
            let body = crate::integration_types::DnsPolicyCreateUpdate {
                policy_type: policy_type_str.to_owned(),
                enabled: req.enabled,
                fields,
            };
            ic.create_dns_policy(&sid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::UpdateDnsPolicy { id, update } => {
            let (ic, sid) = require_integration(integration, site_id, "UpdateDnsPolicy")?;
            let uuid = require_uuid(&id)?;
            let existing = ic.get_dns_policy(&sid, &uuid).await?;
            let fields = build_update_dns_policy_fields(&existing, &update)?;

            let body = crate::integration_types::DnsPolicyCreateUpdate {
                policy_type: existing.policy_type,
                enabled: update.enabled.unwrap_or(existing.enabled),
                fields,
            };
            ic.update_dns_policy(&sid, &uuid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteDnsPolicy { id } => {
            let (ic, sid) = require_integration(integration, site_id, "DeleteDnsPolicy")?;
            let uuid = require_uuid(&id)?;
            ic.delete_dns_policy(&sid, &uuid).await?;
            Ok(CommandResult::Ok)
        }
        Command::CreateTrafficMatchingList(req) => {
            let (ic, sid) = require_integration(integration, site_id, "CreateTrafficMatchingList")?;
            let mut fields = serde_json::Map::new();
            fields.insert(
                "items".into(),
                serde_json::Value::Array(traffic_matching_list_items(
                    &req.entries,
                    req.raw_items.as_deref(),
                )),
            );
            if let Some(desc) = req.description {
                fields.insert("description".into(), serde_json::Value::String(desc));
            }
            let body = crate::integration_types::TrafficMatchingListCreateUpdate {
                name: req.name,
                list_type: req.list_type,
                fields,
            };
            ic.create_traffic_matching_list(&sid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::UpdateTrafficMatchingList { id, update } => {
            let (ic, sid) = require_integration(integration, site_id, "UpdateTrafficMatchingList")?;
            let uuid = require_uuid(&id)?;
            let existing = ic.get_traffic_matching_list(&sid, &uuid).await?;
            let mut fields = serde_json::Map::new();
            let entries = if let Some(raw_items) = update.raw_items.as_deref() {
                serde_json::Value::Array(raw_items.to_vec())
            } else if let Some(new_entries) = &update.entries {
                serde_json::Value::Array(traffic_matching_list_items(new_entries, None))
            } else if let Some(existing_entries) = existing.extra.get("items") {
                existing_entries.clone()
            } else if let Some(existing_entries) = existing.extra.get("entries") {
                existing_entries.clone()
            } else {
                serde_json::Value::Array(Vec::new())
            };
            fields.insert("items".into(), entries);
            if let Some(desc) = update.description {
                fields.insert("description".into(), serde_json::Value::String(desc));
            } else if let Some(existing_desc) = existing.extra.get("description") {
                fields.insert("description".into(), existing_desc.clone());
            }
            let body = crate::integration_types::TrafficMatchingListCreateUpdate {
                name: update.name.unwrap_or(existing.name),
                list_type: existing.list_type,
                fields,
            };
            ic.update_traffic_matching_list(&sid, &uuid, &body).await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteTrafficMatchingList { id } => {
            let (ic, sid) = require_integration(integration, site_id, "DeleteTrafficMatchingList")?;
            let uuid = require_uuid(&id)?;
            ic.delete_traffic_matching_list(&sid, &uuid).await?;
            Ok(CommandResult::Ok)
        }
        _ => unreachable!("policy::route received non-policy command"),
    }
}
