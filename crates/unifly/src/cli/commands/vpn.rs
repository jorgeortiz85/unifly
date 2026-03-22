//! VPN command handlers.

use tabled::Tabled;
use unifly_api::{Controller, VpnServer, VpnTunnel};

use crate::cli::args::{GlobalOpts, VpnArgs, VpnCommand};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

// ── Table rows ──────────────────────────────────────────────────────

#[derive(Tabled)]
struct VpnServerRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    server_type: String,
    #[tabled(rename = "Enabled")]
    enabled: String,
}

fn vpn_server_row(s: &VpnServer, p: &output::Painter) -> VpnServerRow {
    VpnServerRow {
        id: p.id(&s.id.to_string()),
        name: p.name(&s.name.clone().unwrap_or_default()),
        server_type: p.muted(&s.server_type),
        enabled: s.enabled.map_or_else(
            || p.muted("-"),
            |e| p.enabled(e),
        ),
    }
}

#[derive(Tabled)]
struct VpnTunnelRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    tunnel_type: String,
    #[tabled(rename = "Enabled")]
    enabled: String,
}

fn vpn_tunnel_row(t: &VpnTunnel, p: &output::Painter) -> VpnTunnelRow {
    VpnTunnelRow {
        id: p.id(&t.id.to_string()),
        name: p.name(&t.name.clone().unwrap_or_default()),
        tunnel_type: p.muted(&t.tunnel_type),
        enabled: t.enabled.map_or_else(
            || p.muted("-"),
            |e| p.enabled(e),
        ),
    }
}

// ── Handler ─────────────────────────────────────────────────────────

pub async fn handle(
    controller: &Controller,
    args: VpnArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let p = output::Painter::new(global);

    match args.command {
        VpnCommand::Servers(list) => {
            let servers = util::apply_list_args(
                controller.list_vpn_servers().await?,
                &list,
                util::matches_json_filter,
            );
            let out = output::render_list(
                &global.output,
                &servers,
                |s| vpn_server_row(s, &p),
                |s| s.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        VpnCommand::Tunnels(list) => {
            let tunnels = util::apply_list_args(
                controller.list_vpn_tunnels().await?,
                &list,
                util::matches_json_filter,
            );
            let out = output::render_list(
                &global.output,
                &tunnels,
                |t| vpn_tunnel_row(t, &p),
                |t| t.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }
    }
}
