//! Firewall command handlers (policies + zones).

use std::sync::Arc;

use tabled::Tabled;
use unifly_api::model::{FirewallAction as ModelFirewallAction, FirewallPolicy, FirewallZone};
use unifly_api::{
    Command as CoreCommand, Controller, CreateFirewallPolicyRequest, CreateFirewallZoneRequest,
    EntityId, TrafficFilterSpec, UpdateFirewallPolicyRequest, UpdateFirewallZoneRequest,
};

use crate::cli::args::{
    FirewallAction, FirewallArgs, FirewallCommand, FirewallPoliciesCommand, FirewallZonesCommand,
    GlobalOpts,
};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

fn map_fw_action(a: &FirewallAction) -> ModelFirewallAction {
    match a {
        FirewallAction::Allow => ModelFirewallAction::Allow,
        FirewallAction::Block => ModelFirewallAction::Block,
        FirewallAction::Reject => ModelFirewallAction::Reject,
    }
}

// ── Policy table row ────────────────────────────────────────────────

#[derive(Tabled)]
struct PolicyRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Action")]
    action: String,
    #[tabled(rename = "Enabled")]
    enabled: String,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Destination")]
    destination: String,
}

fn policy_row(pol: &Arc<FirewallPolicy>, p: &output::Painter) -> PolicyRow {
    let src = pol
        .source_summary
        .as_deref()
        .or_else(|| pol.source.zone_id.as_ref().map(|_| "zone-only"))
        .unwrap_or("-");
    let dst = pol
        .destination_summary
        .as_deref()
        .or_else(|| pol.destination.zone_id.as_ref().map(|_| "zone-only"))
        .unwrap_or("-");
    PolicyRow {
        id: p.id(&pol.id.to_string()),
        name: p.name(&pol.name),
        action: p.action(&format!("{:?}", pol.action)),
        enabled: p.enabled(pol.enabled),
        source: p.muted(src),
        destination: p.muted(dst),
    }
}

fn policy_detail(p: &Arc<FirewallPolicy>) -> String {
    let mut lines = vec![
        format!("ID:          {}", p.id),
        format!("Name:        {}", p.name),
        format!("Description: {}", p.description.as_deref().unwrap_or("-")),
        format!("Enabled:     {}", p.enabled),
        format!("Action:      {:?}", p.action),
        format!("IP Version:  {:?}", p.ip_version),
        format!(
            "Src Zone:    {}",
            p.source
                .zone_id
                .as_ref()
                .map_or_else(|| "-".into(), ToString::to_string)
        ),
    ];
    if let Some(ref filter) = p.source.filter {
        lines.push(format!("Src Filter:  {}", filter.summary()));
    }
    lines.push(format!(
        "Dst Zone:    {}",
        p.destination
            .zone_id
            .as_ref()
            .map_or_else(|| "-".into(), ToString::to_string)
    ));
    if let Some(ref filter) = p.destination.filter {
        lines.push(format!("Dst Filter:  {}", filter.summary()));
    }
    lines.push(format!(
        "States:      {}",
        if p.connection_states.is_empty() {
            "any".into()
        } else {
            p.connection_states.join(", ")
        }
    ));
    lines.push(format!("Logging:     {}", p.logging_enabled));
    lines.join("\n")
}

// ── Zone table row ──────────────────────────────────────────────────

#[derive(Tabled)]
struct ZoneRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Networks")]
    network_count: String,
}

fn zone_row(z: &Arc<FirewallZone>, p: &output::Painter) -> ZoneRow {
    ZoneRow {
        id: p.id(&z.id.to_string()),
        name: p.name(&z.name),
        network_count: p.number(&z.network_ids.len().to_string()),
    }
}

fn zone_detail(z: &Arc<FirewallZone>) -> String {
    let nets = z
        .network_ids
        .iter()
        .map(|id| format!("  - {id}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "ID:       {}\nName:     {}\nNetworks:\n{}",
        z.id, z.name, nets
    )
}

// ── Handler ─────────────────────────────────────────────────────────

pub async fn handle(
    controller: &Controller,
    args: FirewallArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    util::ensure_integration_access(controller, "firewall").await?;
    let p = output::Painter::new(global);

    match args.command {
        FirewallCommand::Policies(pargs) => {
            handle_policies(controller, pargs.command, global, &p).await
        }
        FirewallCommand::Zones(zargs) => handle_zones(controller, zargs.command, global, &p).await,
    }
}

#[allow(clippy::too_many_lines)]
async fn handle_policies(
    controller: &Controller,
    cmd: FirewallPoliciesCommand,
    global: &GlobalOpts,
    p: &output::Painter,
) -> Result<(), CliError> {
    match cmd {
        FirewallPoliciesCommand::List(list) => {
            let all = controller.firewall_policies_snapshot();
            let snap = util::apply_list_args(all.iter().cloned(), &list, |pol, filter| {
                util::matches_json_filter(pol, filter)
            });
            let out = output::render_list(
                &global.output,
                &snap,
                |pol| policy_row(pol, p),
                |pol| pol.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        FirewallPoliciesCommand::Get { id } => {
            let snap = controller.firewall_policies_snapshot();
            let found = snap.iter().find(|p| p.id.to_string() == id);
            match found {
                Some(p) => {
                    let out = output::render_single(&global.output, p, policy_detail, |p| {
                        p.id.to_string()
                    });
                    output::print_output(&out, global.quiet);
                }
                None => {
                    return Err(CliError::NotFound {
                        resource_type: "firewall policy".into(),
                        identifier: id,
                        list_command: "firewall policies list".into(),
                    });
                }
            }
            Ok(())
        }

        FirewallPoliciesCommand::Create {
            from_file,
            name,
            action,
            source_zone,
            dest_zone,
            enabled,
            description,
            logging,
            src_network,
            src_ip,
            src_port,
            dst_network,
            dst_ip,
            dst_port,
            states,
            ip_version,
        } => {
            let req = if let Some(ref path) = from_file {
                serde_json::from_value(util::read_json_file(path)?)?
            } else {
                CreateFirewallPolicyRequest {
                    name: name.unwrap_or_default(),
                    action: action
                        .as_ref()
                        .map_or(ModelFirewallAction::Block, map_fw_action),
                    source_zone_id: EntityId::from(source_zone.unwrap_or_default()),
                    destination_zone_id: EntityId::from(dest_zone.unwrap_or_default()),
                    enabled,
                    logging_enabled: logging,
                    description,
                    ip_version,
                    connection_states: states,
                    source_filter: build_filter_spec(src_network, src_ip, src_port),
                    destination_filter: build_filter_spec(dst_network, dst_ip, dst_port),
                }
            };

            controller
                .execute(CoreCommand::CreateFirewallPolicy(req))
                .await?;
            if !global.quiet {
                eprintln!("Firewall policy created");
            }
            Ok(())
        }

        FirewallPoliciesCommand::Update {
            id,
            from_file,
            src_network,
            src_ip,
            src_port,
            dst_network,
            dst_ip,
            dst_port,
            states,
            ip_version,
        } => {
            if from_file.is_none()
                && src_network.is_none()
                && src_ip.is_none()
                && src_port.is_none()
                && dst_network.is_none()
                && dst_ip.is_none()
                && dst_port.is_none()
                && states.is_none()
                && ip_version.is_none()
            {
                return Err(CliError::Validation {
                    field: "update".into(),
                    reason: "at least one update flag or --from-file is required".into(),
                });
            }
            let update = if let Some(ref path) = from_file {
                serde_json::from_value(util::read_json_file(path)?)?
            } else {
                UpdateFirewallPolicyRequest {
                    source_filter: build_filter_spec(src_network, src_ip, src_port),
                    destination_filter: build_filter_spec(dst_network, dst_ip, dst_port),
                    connection_states: states,
                    ip_version,
                    ..UpdateFirewallPolicyRequest::default()
                }
            };
            let eid = EntityId::from(id);
            controller
                .execute(CoreCommand::UpdateFirewallPolicy { id: eid, update })
                .await?;
            if !global.quiet {
                eprintln!("Firewall policy updated");
            }
            Ok(())
        }

        FirewallPoliciesCommand::Patch {
            id,
            enabled,
            logging,
        } => {
            if enabled.is_none() && logging.is_none() {
                return Err(CliError::Validation {
                    field: "patch".into(),
                    reason: "at least one of --enabled or --logging is required".into(),
                });
            }
            let eid = EntityId::from(id);
            controller
                .execute(CoreCommand::PatchFirewallPolicy {
                    id: eid,
                    enabled,
                    logging,
                })
                .await?;
            if !global.quiet {
                let mut parts = Vec::new();
                if let Some(e) = enabled {
                    parts.push(if e { "enabled" } else { "disabled" });
                }
                if let Some(l) = logging {
                    parts.push(if l {
                        "logging enabled"
                    } else {
                        "logging disabled"
                    });
                }
                eprintln!("Firewall policy {}", parts.join(", "));
            }
            Ok(())
        }

        FirewallPoliciesCommand::Delete { id } => {
            let eid = EntityId::from(id.clone());
            if !util::confirm(&format!("Delete firewall policy {id}?"), global.yes)? {
                return Ok(());
            }
            controller
                .execute(CoreCommand::DeleteFirewallPolicy { id: eid })
                .await?;
            if !global.quiet {
                eprintln!("Firewall policy deleted");
            }
            Ok(())
        }

        FirewallPoliciesCommand::Reorder {
            source_zone,
            dest_zone,
            get,
            set,
        } => {
            if let Some(ids) = set {
                let zone_pair = (EntityId::from(source_zone), EntityId::from(dest_zone));
                let ordered_ids: Vec<EntityId> = ids.into_iter().map(EntityId::from).collect();
                controller
                    .execute(CoreCommand::ReorderFirewallPolicies {
                        zone_pair,
                        ordered_ids,
                    })
                    .await?;
                if !global.quiet {
                    eprintln!("Firewall policy order updated");
                }
            } else {
                let _ = get;
                let ordering = controller.get_firewall_policy_ordering().await?;
                let out = match &global.output {
                    crate::cli::args::OutputFormat::Table
                    | crate::cli::args::OutputFormat::Plain => {
                        let before = ordering
                            .before_system_defined
                            .iter()
                            .map(|id| format!("  - {id}"))
                            .collect::<Vec<_>>()
                            .join("\n");
                        let after = ordering
                            .after_system_defined
                            .iter()
                            .map(|id| format!("  - {id}"))
                            .collect::<Vec<_>>()
                            .join("\n");
                        format!(
                            "Before System Defined:\n{}\n\nAfter System Defined:\n{}",
                            if before.is_empty() {
                                "  (none)"
                            } else {
                                &before
                            },
                            if after.is_empty() { "  (none)" } else { &after }
                        )
                    }
                    crate::cli::args::OutputFormat::Json => {
                        serde_json::to_string_pretty(&ordering).unwrap_or_default()
                    }
                    crate::cli::args::OutputFormat::JsonCompact => {
                        serde_json::to_string(&ordering).unwrap_or_default()
                    }
                    crate::cli::args::OutputFormat::Yaml => {
                        serde_yml::to_string(&ordering).unwrap_or_default()
                    }
                };
                output::print_output(&out, global.quiet);
            }
            Ok(())
        }
    }
}

async fn handle_zones(
    controller: &Controller,
    cmd: FirewallZonesCommand,
    global: &GlobalOpts,
    p: &output::Painter,
) -> Result<(), CliError> {
    match cmd {
        FirewallZonesCommand::List(list) => {
            let all = controller.firewall_zones_snapshot();
            let snap = util::apply_list_args(all.iter().cloned(), &list, |z, filter| {
                util::matches_json_filter(z, filter)
            });
            let out = output::render_list(
                &global.output,
                &snap,
                |z| zone_row(z, p),
                |z| z.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        FirewallZonesCommand::Get { id } => {
            let snap = controller.firewall_zones_snapshot();
            let found = snap.iter().find(|z| z.id.to_string() == id);
            match found {
                Some(z) => {
                    let out =
                        output::render_single(&global.output, z, zone_detail, |z| z.id.to_string());
                    output::print_output(&out, global.quiet);
                }
                None => {
                    return Err(CliError::NotFound {
                        resource_type: "firewall zone".into(),
                        identifier: id,
                        list_command: "firewall zones list".into(),
                    });
                }
            }
            Ok(())
        }

        FirewallZonesCommand::Create { name, networks } => {
            let network_ids = networks
                .unwrap_or_default()
                .into_iter()
                .map(EntityId::from)
                .collect();
            let req = CreateFirewallZoneRequest {
                name,
                description: None,
                network_ids,
            };
            controller
                .execute(CoreCommand::CreateFirewallZone(req))
                .await?;
            if !global.quiet {
                eprintln!("Firewall zone created");
            }
            Ok(())
        }

        FirewallZonesCommand::Update { id, name, networks } => {
            if name.is_none() && networks.is_none() {
                return Err(CliError::Validation {
                    field: "update".into(),
                    reason: "at least one of --name or --networks is required".into(),
                });
            }
            let eid = EntityId::from(id);
            let update = UpdateFirewallZoneRequest {
                name,
                description: None,
                network_ids: networks.map(|ns| ns.into_iter().map(EntityId::from).collect()),
            };
            controller
                .execute(CoreCommand::UpdateFirewallZone { id: eid, update })
                .await?;
            if !global.quiet {
                eprintln!("Firewall zone updated");
            }
            Ok(())
        }

        FirewallZonesCommand::Delete { id } => {
            let eid = EntityId::from(id.clone());
            if !util::confirm(&format!("Delete firewall zone {id}?"), global.yes)? {
                return Ok(());
            }
            controller
                .execute(CoreCommand::DeleteFirewallZone { id: eid })
                .await?;
            if !global.quiet {
                eprintln!("Firewall zone deleted");
            }
            Ok(())
        }
    }
}

/// Build a `TrafficFilterSpec` from the CLI filter flags.
/// Priority: network > ip > port (first one specified wins).
fn build_filter_spec(
    networks: Option<Vec<String>>,
    ips: Option<Vec<String>>,
    ports: Option<Vec<String>>,
) -> Option<TrafficFilterSpec> {
    if let Some(nets) = networks {
        Some(TrafficFilterSpec::Network {
            network_ids: nets,
            match_opposite: false,
        })
    } else if let Some(addrs) = ips {
        Some(TrafficFilterSpec::IpAddress {
            addresses: addrs,
            match_opposite: false,
        })
    } else {
        ports.map(|p| TrafficFilterSpec::Port {
            ports: p,
            match_opposite: false,
        })
    }
}
