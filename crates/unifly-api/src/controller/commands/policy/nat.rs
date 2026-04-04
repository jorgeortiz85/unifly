use serde_json::json;

use crate::command::{Command, CommandResult};
use crate::core_error::CoreError;

use super::super::{CommandContext, require_legacy};

pub(super) async fn route(ctx: &CommandContext, cmd: Command) -> Result<CommandResult, CoreError> {
    let legacy = ctx.legacy.as_ref();

    match cmd {
        Command::CreateNatPolicy(req) => {
            let legacy = require_legacy(legacy)?;

            let nat_type = match req.nat_type.to_lowercase().as_str() {
                "masquerade" => "MASQUERADE",
                "source" | "source_nat" | "snat" => "SNAT",
                _ => "DNAT",
            };

            let protocol = req
                .protocol
                .as_deref()
                .map(|p| match p.to_lowercase().as_str() {
                    "tcp" => "tcp",
                    "udp" => "udp",
                    "tcp_udp" | "tcp_and_udp" => "tcp_udp",
                    _ => "all",
                });

            // Build v2 API body matching the controller's expected format
            let mut body = json!({
                "description": req.name,
                "enabled": req.enabled,
                "type": nat_type,
                "ip_version": "IPV4",
                "is_predefined": false,
                "rule_index": 0,
                "setting_preference": "manual",
                "logging": false,
                "exclude": false,
                "pppoe_use_base_interface": false,
            });

            if let Some(proto) = protocol {
                body["protocol"] = json!(proto);
            }

            // Translated address — DNAT/SNAT use ip_address, masquerade
            // uses the interface's own IP automatically.
            if let Some(addr) = &req.translated_address {
                body["ip_address"] = json!(addr);
            }

            // Translated port (top-level "port" in v2 schema)
            if let Some(port) = &req.translated_port {
                body["port"] = json!(port);
            }

            // DNAT matches traffic entering an interface (in_interface);
            // SNAT/masquerade matches traffic leaving one (out_interface).
            if let Some(iface) = &req.interface_id {
                let key = if nat_type == "DNAT" {
                    "in_interface"
                } else {
                    "out_interface"
                };
                body[key] = json!(iface.to_string());
            }

            // Source filter
            body["source_filter"] =
                build_filter(req.src_address.as_deref(), req.src_port.as_deref());

            // Destination filter
            body["destination_filter"] =
                build_filter(req.dst_address.as_deref(), req.dst_port.as_deref());

            legacy.create_nat_rule(&body).await?;
            Ok(CommandResult::Ok)
        }
        Command::DeleteNatPolicy { id } => {
            let legacy = require_legacy(legacy)?;
            legacy.delete_nat_rule(&id.to_string()).await?;
            Ok(CommandResult::Ok)
        }
        _ => unreachable!("nat::route received non-NAT command"),
    }
}

/// Build a v2 NAT filter object (source_filter or destination_filter).
fn build_filter(address: Option<&str>, port: Option<&str>) -> serde_json::Value {
    match (address, port) {
        (Some(addr), Some(p)) => json!({
            "filter_type": "ADDRESS_AND_PORT",
            "address": addr,
            "port": p,
            "firewall_group_ids": [],
            "invert_address": false,
            "invert_port": false,
        }),
        (Some(addr), None) => json!({
            "filter_type": "ADDRESS",
            "address": addr,
            "firewall_group_ids": [],
            "invert_address": false,
            "invert_port": false,
        }),
        (None, Some(p)) => json!({
            "filter_type": "PORT",
            "port": p,
            "firewall_group_ids": [],
            "invert_address": false,
            "invert_port": false,
        }),
        (None, None) => json!({
            "filter_type": "NONE",
            "firewall_group_ids": [],
            "invert_address": false,
            "invert_port": false,
        }),
    }
}
