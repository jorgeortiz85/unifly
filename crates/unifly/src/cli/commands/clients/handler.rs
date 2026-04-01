use std::net::Ipv4Addr;
use std::sync::Arc;

use unifly_api::{Client, Command as CoreCommand, Controller, EntityId, MacAddress};

use crate::cli::args::{ClientsArgs, ClientsCommand, GlobalOpts};
use crate::cli::commands::util;
use crate::cli::error::CliError;
use crate::cli::output;

use super::render::{Reservation, client_row, detail, reservation_row};
use super::resolve::resolve_network;

fn find_client(controller: &Controller, needle: &str) -> Option<Arc<Client>> {
    controller
        .clients_snapshot()
        .iter()
        .find(|client| client.id.to_string() == needle || client.mac.to_string() == needle)
        .cloned()
}

fn matches_find_query(client: &Client, query: &str) -> bool {
    let fields = [
        client.ip.map(|ip| ip.to_string()),
        client.name.clone(),
        client.hostname.clone(),
        Some(client.mac.to_string()),
    ];
    fields.iter().any(|field| {
        field
            .as_ref()
            .is_some_and(|value| value.to_lowercase().contains(query))
    })
}

#[allow(clippy::too_many_lines)]
pub(super) async fn handle(
    controller: &Controller,
    args: ClientsArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);

    match args.command {
        ClientsCommand::List(list) => {
            let all = controller.clients_snapshot();
            let snapshot = util::apply_list_args(all.iter().cloned(), &list, |client, filter| {
                util::matches_json_filter(client, filter)
            });
            let out = output::render_list(
                &global.output,
                &snapshot,
                |client| client_row(client, &painter),
                |client| client.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        ClientsCommand::Find { query } => {
            let normalized_query = query.to_lowercase();
            let matches: Vec<_> = controller
                .clients_snapshot()
                .iter()
                .filter(|client| matches_find_query(client, &normalized_query))
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
                |client| client_row(client, &painter),
                |client| client.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        ClientsCommand::Get { client } => {
            match find_client(controller, &client) {
                Some(client) => {
                    let out = output::render_single(&global.output, &client, detail, |client| {
                        client.id.to_string()
                    });
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

        ClientsCommand::Reservations(list) => {
            let users = controller.list_users().await?;
            let reservations: Vec<Reservation> = users
                .iter()
                .filter(|u| u.use_fixedip.unwrap_or(false))
                .map(Reservation::from)
                .collect();
            let snapshot =
                util::apply_list_args(reservations.into_iter(), &list, |res, filter| {
                    util::matches_json_filter(res, filter)
                });
            let out = output::render_list(
                &global.output,
                &snapshot,
                |res| reservation_row(res, &painter),
                |res| res.mac.clone(),
            );
            output::print_output(&out, global.quiet);
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
