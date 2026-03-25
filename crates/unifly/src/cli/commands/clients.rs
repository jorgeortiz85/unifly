//! Client command handlers.

use std::net::Ipv4Addr;
use std::sync::Arc;

use tabled::Tabled;
use unifly_api::{Client, Command as CoreCommand, Controller, EntityId, MacAddress};

use crate::cli::args::{ClientsArgs, ClientsCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

// ── Table row ───────────────────────────────────────────────────────

#[derive(Tabled)]
struct ClientRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "IP")]
    ip: String,
    #[tabled(rename = "Type")]
    ctype: String,
    #[tabled(rename = "Uplink")]
    uplink: String,
}

fn client_row(c: &Arc<Client>, p: &output::Painter) -> ClientRow {
    let name = c
        .name
        .clone()
        .or_else(|| c.hostname.clone())
        .unwrap_or_default();
    ClientRow {
        name: p.name(&name),
        ip: p.ip(&c.ip.map(|ip| ip.to_string()).unwrap_or_default()),
        ctype: p.muted(&format!("{:?}", c.client_type)),
        uplink: p.mac(
            &c.uplink_device_mac
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
        ),
    }
}

fn detail(c: &Arc<Client>) -> String {
    let mut lines = vec![
        format!("ID:        {}", c.id),
        format!("Name:      {}", c.name.as_deref().unwrap_or("-")),
        format!("Hostname:  {}", c.hostname.as_deref().unwrap_or("-")),
        format!("MAC:       {}", c.mac),
        format!(
            "IP:        {}",
            c.ip.map_or_else(|| "-".into(), |ip| ip.to_string())
        ),
        format!("Type:      {:?}", c.client_type),
        format!("Guest:     {}", c.is_guest),
        format!("Blocked:   {}", c.blocked),
    ];
    if c.use_fixedip {
        lines.push(format!(
            "Fixed IP:  {}",
            c.fixed_ip.map_or("-".into(), |ip| ip.to_string())
        ));
    }
    if let Some(ref w) = c.wireless {
        lines.push(format!("SSID:      {}", w.ssid.as_deref().unwrap_or("-")));
        if let Some(sig) = w.signal_dbm {
            lines.push(format!("Signal:    {sig} dBm"));
        }
    }
    if let Some(os) = &c.os_name {
        lines.push(format!("OS:        {os}"));
    }
    lines.join("\n")
}

// ── Handler ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
pub async fn handle(
    controller: &Controller,
    args: ClientsArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let p = output::Painter::new(global);

    match args.command {
        ClientsCommand::List(list) => {
            let all = controller.clients_snapshot();
            let snap = util::apply_list_args(all.iter().cloned(), &list, |c, filter| {
                util::matches_json_filter(c, filter)
            });
            let out = output::render_list(
                &global.output,
                &snap,
                |c| client_row(c, &p),
                |c| c.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        ClientsCommand::Find { query } => {
            let q = query.to_lowercase();
            let all = controller.clients_snapshot();
            let matches: Vec<_> = all
                .iter()
                .filter(|c| {
                    let fields = [
                        c.ip.map(|ip| ip.to_string()),
                        c.name.clone(),
                        c.hostname.clone(),
                        Some(c.mac.to_string()),
                    ];
                    fields
                        .iter()
                        .any(|f| f.as_ref().is_some_and(|v| v.to_lowercase().contains(&q)))
                })
                .cloned()
                .collect();
            if matches.is_empty() {
                return Err(CliError::NotFound {
                    resource_type: "client".into(),
                    identifier: query,
                    list_command: "clients list".into(),
                });
            }
            let out = output::render_list(
                &global.output,
                &matches,
                |c| client_row(c, &p),
                |c| c.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        ClientsCommand::Get { client } => {
            let snap = controller.clients_snapshot();
            let found = snap
                .iter()
                .find(|c| c.id.to_string() == client || c.mac.to_string() == client);
            match found {
                Some(c) => {
                    let out =
                        output::render_single(&global.output, c, detail, |c| c.id.to_string());
                    output::print_output(&out, global.quiet);
                }
                None => {
                    return Err(CliError::NotFound {
                        resource_type: "client".into(),
                        identifier: client,
                        list_command: "clients list".into(),
                    });
                }
            }
            Ok(())
        }

        ClientsCommand::Authorize {
            client,
            minutes,
            data_limit_mb,
            rx_limit_kbps,
            tx_limit_kbps,
        } => {
            let client_id = EntityId::from(client);
            controller
                .execute(CoreCommand::AuthorizeGuest {
                    client_id,
                    time_limit_minutes: Some(minutes),
                    data_limit_mb,
                    rx_rate_kbps: rx_limit_kbps,
                    tx_rate_kbps: tx_limit_kbps,
                })
                .await?;
            if !global.quiet {
                eprintln!("Guest authorized for {minutes} minutes");
            }
            Ok(())
        }

        ClientsCommand::Unauthorize { client } => {
            let client_id = EntityId::from(client);
            controller
                .execute(CoreCommand::UnauthorizeGuest { client_id })
                .await?;
            if !global.quiet {
                eprintln!("Guest authorization revoked");
            }
            Ok(())
        }

        ClientsCommand::Block { mac } => {
            let mac = MacAddress::new(&mac);
            controller.execute(CoreCommand::BlockClient { mac }).await?;
            if !global.quiet {
                eprintln!("Client blocked");
            }
            Ok(())
        }

        ClientsCommand::Unblock { mac } => {
            let mac = MacAddress::new(&mac);
            controller
                .execute(CoreCommand::UnblockClient { mac })
                .await?;
            if !global.quiet {
                eprintln!("Client unblocked");
            }
            Ok(())
        }

        ClientsCommand::Kick { mac } => {
            let mac = MacAddress::new(&mac);
            controller.execute(CoreCommand::KickClient { mac }).await?;
            if !global.quiet {
                eprintln!("Client disconnected");
            }
            Ok(())
        }

        ClientsCommand::Forget { mac } => {
            let mac_addr = MacAddress::new(&mac);
            if !util::confirm(
                &format!("Forget client {mac}? This cannot be undone."),
                global.yes,
            )? {
                return Ok(());
            }
            controller
                .execute(CoreCommand::ForgetClient { mac: mac_addr })
                .await?;
            if !global.quiet {
                eprintln!("Client forgotten");
            }
            Ok(())
        }

        ClientsCommand::SetIp { mac, ip, network } => {
            let ip_addr: Ipv4Addr = ip.parse().map_err(|_| CliError::Validation {
                field: "ip".into(),
                reason: format!("'{ip}' is not a valid IPv4 address"),
            })?;

            let network_id = resolve_network(controller, network.as_deref(), ip_addr)?;
            let mac_addr = MacAddress::new(&mac);

            controller
                .execute(CoreCommand::SetClientFixedIp {
                    mac: mac_addr,
                    ip: ip_addr,
                    network_id,
                })
                .await?;
            if !global.quiet {
                eprintln!("Fixed IP {ip} set for {mac}");
            }
            Ok(())
        }

        ClientsCommand::RemoveIp { mac } => {
            let mac_addr = MacAddress::new(&mac);
            controller
                .execute(CoreCommand::RemoveClientFixedIp { mac: mac_addr })
                .await?;
            if !global.quiet {
                eprintln!("Fixed IP removed for {mac}");
            }
            Ok(())
        }
    }
}

/// Resolve a network by name/ID, or auto-detect from an IP address by
/// matching against known network subnets.
fn resolve_network(
    controller: &Controller,
    name_or_id: Option<&str>,
    ip: Ipv4Addr,
) -> Result<EntityId, CliError> {
    let networks = controller.networks_snapshot();

    if let Some(needle) = name_or_id {
        // Explicit network: match by name or ID
        return networks
            .iter()
            .find(|n| n.name.eq_ignore_ascii_case(needle) || n.id.to_string() == needle)
            .map(|n| n.id.clone())
            .ok_or_else(|| CliError::NotFound {
                resource_type: "network".into(),
                identifier: needle.into(),
                list_command: "networks list".into(),
            });
    }

    // Auto-detect: find the network whose subnet contains the IP
    let ip_u32 = u32::from(ip);
    for net in networks.iter() {
        if let Some(ref subnet_str) = net.subnet
            && let Some((net_addr, prefix)) = parse_cidr(subnet_str)
        {
            let mask = if prefix == 0 {
                0
            } else {
                u32::MAX << (32 - prefix)
            };
            if (ip_u32 & mask) == (u32::from(net_addr) & mask) {
                return Ok(net.id.clone());
            }
        }
    }

    Err(CliError::Validation {
        field: "network".into(),
        reason: format!(
            "could not auto-detect network for IP {ip}; use --network to specify explicitly"
        ),
    })
}

/// Parse "10.4.22.1/24" into (Ipv4Addr, prefix_len).
fn parse_cidr(s: &str) -> Option<(Ipv4Addr, u32)> {
    let (addr_str, prefix_str) = s.split_once('/')?;
    let addr: Ipv4Addr = addr_str.parse().ok()?;
    let prefix: u32 = prefix_str.parse().ok()?;
    Some((addr, prefix))
}
