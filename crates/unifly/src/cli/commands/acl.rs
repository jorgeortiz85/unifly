//! ACL rule command handlers.

use std::sync::Arc;

use tabled::Tabled;
use unifly_api::model::AclRule;
use unifly_api::{
    Command as CoreCommand, Controller, CreateAclRuleRequest, EntityId, UpdateAclRuleRequest,
};

use crate::cli::args::{AclArgs, AclCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

// ── Table row ───────────────────────────────────────────────────────

#[derive(Tabled)]
struct AclRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    rule_type: String,
    #[tabled(rename = "Action")]
    action: String,
    #[tabled(rename = "Enabled")]
    enabled: String,
}

fn acl_row(r: &Arc<AclRule>, p: &output::Painter) -> AclRow {
    AclRow {
        id: p.id(&r.id.to_string()),
        name: p.name(&r.name),
        rule_type: p.muted(&format!("{:?}", r.rule_type)),
        action: p.action(&format!("{:?}", r.action)),
        enabled: p.enabled(r.enabled),
    }
}

fn detail(r: &Arc<AclRule>) -> String {
    [
        format!("ID:      {}", r.id),
        format!("Name:    {}", r.name),
        format!("Enabled: {}", r.enabled),
        format!("Type:    {:?}", r.rule_type),
        format!("Action:  {:?}", r.action),
        format!("Source:  {}", r.source_summary.as_deref().unwrap_or("-")),
        format!(
            "Dest:    {}",
            r.destination_summary.as_deref().unwrap_or("-")
        ),
    ]
    .join("\n")
}

fn render_reorder_ids(output: &crate::cli::args::OutputFormat, ids: &[String]) -> String {
    match output {
        crate::cli::args::OutputFormat::Json => {
            serde_json::to_string_pretty(ids).unwrap_or_default()
        }
        crate::cli::args::OutputFormat::JsonCompact => {
            serde_json::to_string(ids).unwrap_or_default()
        }
        _ => ids.join("\n"),
    }
}

fn acl_rule_type_name(rule_type: &crate::cli::args::AclRuleType) -> &'static str {
    match rule_type {
        crate::cli::args::AclRuleType::Ipv4 => "IP",
        crate::cli::args::AclRuleType::Mac => "MAC",
    }
}

#[allow(clippy::too_many_arguments)]
fn build_acl_create_request(
    name: Option<String>,
    rule_type: Option<crate::cli::args::AclRuleType>,
    action: Option<crate::cli::args::AclAction>,
    source_zone: Option<String>,
    dest_zone: Option<String>,
    protocol: Option<String>,
    source_port: Option<String>,
    destination_port: Option<String>,
) -> Result<CreateAclRuleRequest, CliError> {
    let name = name.ok_or_else(|| CliError::Validation {
        field: "name".into(),
        reason: "ACL create requires --name".into(),
    })?;
    let rule_type = rule_type.ok_or_else(|| CliError::Validation {
        field: "rule_type".into(),
        reason: "ACL create requires --rule-type".into(),
    })?;
    let action = action.ok_or_else(|| CliError::Validation {
        field: "action".into(),
        reason: "ACL create requires --action".into(),
    })?;
    let source_zone_id = EntityId::from(source_zone.ok_or_else(|| CliError::Validation {
        field: "source_zone".into(),
        reason: "ACL create requires --source-zone".into(),
    })?);
    let destination_zone_id = EntityId::from(dest_zone.ok_or_else(|| CliError::Validation {
        field: "dest_zone".into(),
        reason: "ACL create requires --dest-zone".into(),
    })?);

    Ok(CreateAclRuleRequest {
        name,
        rule_type: acl_rule_type_name(&rule_type).into(),
        action: match action {
            crate::cli::args::AclAction::Allow => unifly_api::model::FirewallAction::Allow,
            crate::cli::args::AclAction::Block => unifly_api::model::FirewallAction::Block,
        },
        source_zone_id,
        destination_zone_id,
        description: None,
        protocol,
        source_port,
        destination_port,
        source_filter: None,
        destination_filter: None,
        enforcing_device_filter: None,
        enabled: true,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_acl_update_request(
    name: Option<String>,
    rule_type: Option<crate::cli::args::AclRuleType>,
    action: Option<crate::cli::args::AclAction>,
    source_zone: Option<String>,
    dest_zone: Option<String>,
    protocol: Option<String>,
    source_port: Option<String>,
    destination_port: Option<String>,
    enabled: Option<bool>,
) -> Result<UpdateAclRuleRequest, CliError> {
    let update = UpdateAclRuleRequest {
        name,
        rule_type: rule_type.map(|kind| acl_rule_type_name(&kind).to_owned()),
        action: action.map(|value| match value {
            crate::cli::args::AclAction::Allow => unifly_api::model::FirewallAction::Allow,
            crate::cli::args::AclAction::Block => unifly_api::model::FirewallAction::Block,
        }),
        enabled,
        description: None,
        source_zone_id: source_zone.map(EntityId::from),
        destination_zone_id: dest_zone.map(EntityId::from),
        protocol,
        source_port,
        destination_port,
        source_filter: None,
        destination_filter: None,
        enforcing_device_filter: None,
    };

    let has_changes = update.name.is_some()
        || update.rule_type.is_some()
        || update.action.is_some()
        || update.enabled.is_some()
        || update.source_zone_id.is_some()
        || update.destination_zone_id.is_some()
        || update.protocol.is_some()
        || update.source_port.is_some()
        || update.destination_port.is_some();

    if has_changes {
        Ok(update)
    } else {
        Err(CliError::Validation {
            field: "update".into(),
            reason: "ACL update requires at least one change or --from-file".into(),
        })
    }
}

// ── Handler ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
pub async fn handle(
    controller: &Controller,
    args: AclArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    util::ensure_integration_access(controller, "acl").await?;

    let p = output::Painter::new(global);

    match args.command {
        AclCommand::List(list) => {
            let all = controller.acl_rules_snapshot();
            let snap = util::apply_list_args(all.iter().cloned(), &list, |r, filter| {
                util::matches_json_filter(r, filter)
            });
            let out = output::render_list(
                &global.output,
                &snap,
                |r| acl_row(r, &p),
                |r| r.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        AclCommand::Get { id } => {
            let snap = controller.acl_rules_snapshot();
            let found = snap.iter().find(|r| r.id.to_string() == id);
            match found {
                Some(r) => {
                    let out =
                        output::render_single(&global.output, r, detail, |r| r.id.to_string());
                    output::print_output(&out, global.quiet);
                }
                None => {
                    return Err(CliError::NotFound {
                        resource_type: "ACL rule".into(),
                        identifier: id,
                        list_command: "acl list".into(),
                    });
                }
            }
            Ok(())
        }

        AclCommand::Create {
            from_file,
            name,
            rule_type,
            action,
            source_zone,
            dest_zone,
            protocol,
            source_port,
            destination_port,
        } => {
            let req = if let Some(ref path) = from_file {
                serde_json::from_value(util::read_json_file(path)?)?
            } else {
                build_acl_create_request(
                    name,
                    rule_type,
                    action,
                    source_zone,
                    dest_zone,
                    protocol,
                    source_port,
                    destination_port,
                )?
            };
            controller.execute(CoreCommand::CreateAclRule(req)).await?;
            if !global.quiet {
                eprintln!("ACL rule created");
            }
            Ok(())
        }

        AclCommand::Update {
            id,
            name,
            rule_type,
            action,
            source_zone,
            dest_zone,
            protocol,
            source_port,
            destination_port,
            enabled,
            from_file,
        } => {
            let update = if let Some(ref path) = from_file {
                serde_json::from_value(util::read_json_file(path)?)?
            } else {
                build_acl_update_request(
                    name,
                    rule_type,
                    action,
                    source_zone,
                    dest_zone,
                    protocol,
                    source_port,
                    destination_port,
                    enabled,
                )?
            };
            let eid = EntityId::from(id);
            controller
                .execute(CoreCommand::UpdateAclRule { id: eid, update })
                .await?;
            if !global.quiet {
                eprintln!("ACL rule updated");
            }
            Ok(())
        }

        AclCommand::Delete { id } => {
            let eid = EntityId::from(id.clone());
            if !util::confirm(&format!("Delete ACL rule {id}?"), global.yes)? {
                return Ok(());
            }
            controller
                .execute(CoreCommand::DeleteAclRule { id: eid })
                .await?;
            if !global.quiet {
                eprintln!("ACL rule deleted");
            }
            Ok(())
        }

        AclCommand::Reorder { get, set } => {
            if let Some(ids) = set {
                let ordered_ids: Vec<EntityId> = ids.into_iter().map(EntityId::from).collect();
                controller
                    .execute(CoreCommand::ReorderAclRules { ordered_ids })
                    .await?;
                if !global.quiet {
                    eprintln!("ACL rule order updated");
                }
            } else {
                let _ = get;
                let ordering = controller.get_acl_rule_ordering().await?;
                let ids = ordering
                    .ordered_acl_rule_ids
                    .into_iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>();
                let out = render_reorder_ids(&global.output, &ids);
                output::print_output(&out, global.quiet);
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_acl_create_request, build_acl_update_request, render_reorder_ids};
    use crate::cli::args::OutputFormat;

    #[test]
    fn render_reorder_ids_keeps_json_compact_compact() {
        let rendered = render_reorder_ids(&OutputFormat::JsonCompact, &["a".into(), "b".into()]);
        assert_eq!(rendered, "[\"a\",\"b\"]");
    }

    #[test]
    fn render_reorder_ids_pretty_prints_json() {
        let rendered = render_reorder_ids(&OutputFormat::Json, &["a".into()]);
        assert!(rendered.contains('\n'));
    }

    #[test]
    fn build_acl_create_request_uses_api_rule_type_names() {
        let create = build_acl_create_request(
            Some("Block cameras".into()),
            Some(crate::cli::args::AclRuleType::Mac),
            Some(crate::cli::args::AclAction::Block),
            Some("src-zone".into()),
            Some("dst-zone".into()),
            Some("TCP".into()),
            Some("1234".into()),
            Some("443".into()),
        )
        .expect("create request should build");

        assert_eq!(create.rule_type, "MAC");
        assert_eq!(create.source_zone_id.to_string(), "src-zone");
        assert_eq!(create.destination_zone_id.to_string(), "dst-zone");
    }

    #[test]
    fn build_acl_update_request_accepts_inline_fields() {
        let update = build_acl_update_request(
            Some("Updated".into()),
            Some(crate::cli::args::AclRuleType::Ipv4),
            Some(crate::cli::args::AclAction::Allow),
            Some("src-zone".into()),
            Some("dst-zone".into()),
            Some("UDP".into()),
            Some("53".into()),
            Some("5353".into()),
            Some(false),
        )
        .expect("update request should build");

        assert_eq!(update.rule_type.as_deref(), Some("IP"));
        assert_eq!(update.protocol.as_deref(), Some("UDP"));
        assert_eq!(
            update
                .source_zone_id
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("src-zone")
        );
        assert_eq!(
            update
                .destination_zone_id
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("dst-zone")
        );
        assert_eq!(update.enabled, Some(false));
    }

    #[test]
    fn build_acl_update_request_rejects_empty_inline_update() {
        let err = build_acl_update_request(None, None, None, None, None, None, None, None, None)
            .expect_err("empty update should fail");
        match err {
            crate::cli::error::CliError::Validation { field, .. } => assert_eq!(field, "update"),
            other => panic!("expected validation error, got {other:?}"),
        }
    }
}
