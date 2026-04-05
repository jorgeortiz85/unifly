//! VPN command handlers.

use tabled::Tabled;
use unifly_api::{Controller, EntityId, HealthSummary, IpsecSa, VpnServer, VpnTunnel};

use crate::cli::args::{
    GlobalOpts, OutputFormat, VpnArgs, VpnCommand, VpnServersArgs, VpnServersCommand,
    VpnTunnelsArgs, VpnTunnelsCommand,
};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

#[derive(Tabled)]
struct VpnServerRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    server_type: String,
    #[tabled(rename = "Subnet")]
    subnet: String,
    #[tabled(rename = "Port")]
    port: String,
    #[tabled(rename = "Protocol")]
    protocol: String,
    #[tabled(rename = "Enabled")]
    enabled: String,
}

#[derive(Tabled)]
struct VpnTunnelRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    tunnel_type: String,
    #[tabled(rename = "Peer")]
    peer: String,
    #[tabled(rename = "IKE")]
    ike: String,
    #[tabled(rename = "Enabled")]
    enabled: String,
}

#[derive(Tabled)]
struct IpsecSaRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Remote IP")]
    remote_ip: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "TX Bytes")]
    tx: String,
    #[tabled(rename = "RX Bytes")]
    rx: String,
    #[tabled(rename = "Uptime")]
    uptime: String,
    #[tabled(rename = "IKE")]
    ike: String,
}

fn vpn_server_row(server: &VpnServer, painter: &output::Painter) -> VpnServerRow {
    VpnServerRow {
        id: painter.id(&server.id.to_string()),
        name: painter.name(server.name.as_deref().unwrap_or("-")),
        server_type: painter.muted(&server.server_type),
        subnet: painter.ip(server.subnet.as_deref().unwrap_or("-")),
        port: painter.number(&display_optional(server.port)),
        protocol: painter.muted(server.protocol.as_deref().unwrap_or("-")),
        enabled: server
            .enabled
            .map_or_else(|| painter.muted("-"), |enabled| painter.enabled(enabled)),
    }
}

fn vpn_tunnel_row(tunnel: &VpnTunnel, painter: &output::Painter) -> VpnTunnelRow {
    VpnTunnelRow {
        id: painter.id(&tunnel.id.to_string()),
        name: painter.name(tunnel.name.as_deref().unwrap_or("-")),
        tunnel_type: painter.muted(&tunnel.tunnel_type),
        peer: painter.ip(tunnel.peer_address.as_deref().unwrap_or("-")),
        ike: painter.muted(tunnel.ike_version.as_deref().unwrap_or("-")),
        enabled: tunnel
            .enabled
            .map_or_else(|| painter.muted("-"), |enabled| painter.enabled(enabled)),
    }
}

fn ipsec_sa_row(sa: &IpsecSa, painter: &output::Painter) -> IpsecSaRow {
    let state = sa.state.as_deref().unwrap_or("-");
    IpsecSaRow {
        name: painter.name(sa.name.as_deref().unwrap_or("-")),
        remote_ip: painter.ip(sa.remote_ip.as_deref().unwrap_or("-")),
        state: painter.state(state),
        tx: painter.number(&display_optional(sa.tx_bytes)),
        rx: painter.number(&display_optional(sa.rx_bytes)),
        uptime: painter.muted(&display_optional(
            sa.uptime.map(|value| format!("{value}s")),
        )),
        ike: painter.muted(sa.ike_version.as_deref().unwrap_or("-")),
    }
}

fn server_detail(server: &VpnServer, painter: &output::Painter) -> String {
    let mut lines = vec![
        format!("ID:                {}", painter.id(&server.id.to_string())),
        format!(
            "Name:              {}",
            painter.name(server.name.as_deref().unwrap_or("-"))
        ),
        format!("Type:              {}", painter.muted(&server.server_type)),
        format!(
            "Enabled:           {}",
            server
                .enabled
                .map_or_else(|| painter.muted("-"), |enabled| painter.enabled(enabled))
        ),
        format!(
            "Subnet:            {}",
            painter.ip(server.subnet.as_deref().unwrap_or("-"))
        ),
        format!(
            "Port:              {}",
            painter.number(&display_optional(server.port))
        ),
        format!(
            "WAN IP:            {}",
            painter.ip(server.wan_ip.as_deref().unwrap_or("-"))
        ),
        format!(
            "Connected Clients: {}",
            painter.number(&display_optional(server.connected_clients))
        ),
        format!(
            "Protocol:          {}",
            painter.muted(server.protocol.as_deref().unwrap_or("-"))
        ),
    ];
    append_extra(&mut lines, &server.extra);
    lines.join("\n")
}

fn tunnel_detail(tunnel: &VpnTunnel, painter: &output::Painter) -> String {
    let mut lines = vec![
        format!("ID:             {}", painter.id(&tunnel.id.to_string())),
        format!(
            "Name:           {}",
            painter.name(tunnel.name.as_deref().unwrap_or("-"))
        ),
        format!("Type:           {}", painter.muted(&tunnel.tunnel_type)),
        format!(
            "Enabled:        {}",
            tunnel
                .enabled
                .map_or_else(|| painter.muted("-"), |enabled| painter.enabled(enabled))
        ),
        format!(
            "Peer Address:   {}",
            painter.ip(tunnel.peer_address.as_deref().unwrap_or("-"))
        ),
        format!(
            "Local Subnets:  {}",
            painter.ip(&display_list(&tunnel.local_subnets))
        ),
        format!(
            "Remote Subnets: {}",
            painter.ip(&display_list(&tunnel.remote_subnets))
        ),
        format!(
            "Has PSK:        {}",
            if tunnel.has_psk {
                painter.success("yes")
            } else {
                painter.error("no")
            }
        ),
        format!(
            "IKE Version:    {}",
            painter.muted(tunnel.ike_version.as_deref().unwrap_or("-"))
        ),
    ];
    append_extra(&mut lines, &tunnel.extra);
    lines.join("\n")
}

fn vpn_health_detail(health: &HealthSummary, painter: &output::Painter) -> String {
    let mut lines = vec![
        format!("Subsystem: {}", painter.name(&health.subsystem)),
        format!("Status:    {}", painter.health(&health.status)),
        format!(
            "Devices:   {}",
            painter.number(&display_optional(health.num_adopted))
        ),
        format!(
            "Clients:   {}",
            painter.number(&display_optional(health.num_sta))
        ),
        format!(
            "TX/s:      {}",
            painter.number(&display_optional(health.tx_bytes_r))
        ),
        format!(
            "RX/s:      {}",
            painter.number(&display_optional(health.rx_bytes_r))
        ),
        format!(
            "Latency:   {}",
            health.latency.map_or_else(
                || painter.muted("-"),
                |latency| painter.number(&format!("{latency:.1}"))
            )
        ),
        format!(
            "WAN IP:    {}",
            painter.ip(health.wan_ip.as_deref().unwrap_or("-"))
        ),
        format!(
            "Gateways:  {}",
            painter.ip(&health
                .gateways
                .as_ref()
                .map_or_else(|| "-".into(), |gateways| gateways.join(", ")))
        ),
    ];
    if !health.extra.is_null() {
        lines.push(String::new());
        lines.push("Raw:".into());
        lines.push(serde_json::to_string_pretty(&health.extra).unwrap_or_else(|_| "{}".into()));
    }
    lines.join("\n")
}

pub async fn handle(
    controller: &Controller,
    args: VpnArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);

    match args.command {
        VpnCommand::Servers(args) => handle_servers(controller, args, global, &painter).await,
        VpnCommand::Tunnels(args) => handle_tunnels(controller, args, global, &painter).await,
        VpnCommand::Status => {
            let sas = controller.list_ipsec_sa().await?;
            if sas.is_empty() {
                if !global.quiet && matches!(global.output, OutputFormat::Table) {
                    eprintln!("No active IPsec security associations");
                }
                if matches!(global.output, OutputFormat::Table) {
                    return Ok(());
                }
            }
            let out = output::render_list(
                &global.output,
                &sas,
                |sa| ipsec_sa_row(sa, &painter),
                ipsec_sa_identity,
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }
        VpnCommand::Health => match controller.get_vpn_health() {
            Some(health) => {
                let out = output::render_single(
                    &global.output,
                    &health,
                    |health| vpn_health_detail(health, &painter),
                    |health| health.subsystem.clone(),
                );
                output::print_output(&out, global.quiet);
                Ok(())
            }
            None => Err(CliError::NotFound {
                resource_type: "vpn health".into(),
                identifier: "vpn".into(),
                list_command: "system health".into(),
            }),
        },
    }
}

async fn handle_servers(
    controller: &Controller,
    args: VpnServersArgs,
    global: &GlobalOpts,
    painter: &output::Painter,
) -> Result<(), CliError> {
    util::ensure_integration_access(controller, "vpn servers").await?;

    match args.command {
        Some(VpnServersCommand::Get { id }) => {
            let servers = controller.list_vpn_servers().await?;
            let target_id = EntityId::from(id.clone());
            let server = servers.iter().find(|server| server.id == target_id);
            match server {
                Some(server) => {
                    let out = output::render_single(
                        &global.output,
                        server,
                        |server| server_detail(server, painter),
                        |server| server.id.to_string(),
                    );
                    output::print_output(&out, global.quiet);
                    Ok(())
                }
                None => Err(CliError::NotFound {
                    resource_type: "vpn server".into(),
                    identifier: id,
                    list_command: "vpn servers".into(),
                }),
            }
        }
        None => {
            let servers = util::apply_list_args(
                controller.list_vpn_servers().await?,
                &args.list,
                util::matches_json_filter,
            );
            let out = output::render_list(
                &global.output,
                &servers,
                |server| vpn_server_row(server, painter),
                |server| server.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }
    }
}

async fn handle_tunnels(
    controller: &Controller,
    args: VpnTunnelsArgs,
    global: &GlobalOpts,
    painter: &output::Painter,
) -> Result<(), CliError> {
    util::ensure_integration_access(controller, "vpn tunnels").await?;

    match args.command {
        Some(VpnTunnelsCommand::Get { id }) => {
            let tunnels = controller.list_vpn_tunnels().await?;
            let target_id = EntityId::from(id.clone());
            let tunnel = tunnels.iter().find(|tunnel| tunnel.id == target_id);
            match tunnel {
                Some(tunnel) => {
                    let out = output::render_single(
                        &global.output,
                        tunnel,
                        |tunnel| tunnel_detail(tunnel, painter),
                        |tunnel| tunnel.id.to_string(),
                    );
                    output::print_output(&out, global.quiet);
                    Ok(())
                }
                None => Err(CliError::NotFound {
                    resource_type: "vpn tunnel".into(),
                    identifier: id,
                    list_command: "vpn tunnels".into(),
                }),
            }
        }
        None => {
            let tunnels = util::apply_list_args(
                controller.list_vpn_tunnels().await?,
                &args.list,
                util::matches_json_filter,
            );
            let out = output::render_list(
                &global.output,
                &tunnels,
                |tunnel| vpn_tunnel_row(tunnel, painter),
                |tunnel| tunnel.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }
    }
}

fn display_optional<T: ToString>(value: Option<T>) -> String {
    value.map_or_else(|| "-".into(), |value| value.to_string())
}

fn display_list(values: &[String]) -> String {
    if values.is_empty() {
        "-".into()
    } else {
        values.join(", ")
    }
}

fn append_extra(lines: &mut Vec<String>, extra: &serde_json::Map<String, serde_json::Value>) {
    if extra.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push("Raw:".into());
    lines.push(
        serde_json::to_string_pretty(&serde_json::Value::Object(extra.clone()))
            .unwrap_or_else(|_| "{}".into()),
    );
}

fn ipsec_sa_identity(sa: &IpsecSa) -> String {
    sa.name
        .clone()
        .or_else(|| sa.remote_ip.clone())
        .or_else(|| sa.local_ip.clone())
        .unwrap_or_default()
}
