use std::sync::Arc;

use unifly_api::{Command as CoreCommand, Controller, Device, MacAddress};

use crate::cli::args::{DevicesArgs, DevicesCommand, GlobalOpts};
use crate::cli::commands::util;
use crate::cli::error::CliError;
use crate::cli::output;

use super::render::{
    detail, device_row, device_tag_identity, device_tag_row, pending_device_identity,
    pending_device_row, stats_detail,
};

fn find_device(controller: &Controller, needle: &str) -> Option<Arc<Device>> {
    controller
        .devices_snapshot()
        .iter()
        .find(|candidate| candidate.id.to_string() == needle || candidate.mac.to_string() == needle)
        .cloned()
}

#[allow(clippy::too_many_lines)]
pub(super) async fn handle(
    controller: &Controller,
    args: DevicesArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let painter = output::Painter::new(global);

    match args.command {
        DevicesCommand::List(list) => {
            let all = controller.devices_snapshot();
            let snapshot = util::apply_list_args(all.iter().cloned(), &list, |device, filter| {
                util::matches_json_filter(device, filter)
            });
            let out = output::render_list(
                &global.output,
                &snapshot,
                |device| device_row(device, &painter),
                |device| device.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        DevicesCommand::Get { device } => {
            match find_device(controller, &device) {
                Some(device) => {
                    let out = output::render_single(&global.output, &device, detail, |device| {
                        device.id.to_string()
                    });
                    output::print_output(&out, global.quiet);
                }
                None => {
                    return Err(CliError::NotFound {
                        resource_type: "device".into(),
                        identifier: device,
                        list_command: "devices list".into(),
                    });
                }
            }
            Ok(())
        }

        DevicesCommand::Adopt { mac, ignore_limit } => {
            controller
                .execute(CoreCommand::AdoptDevice {
                    mac: MacAddress::new(&mac),
                    ignore_device_limit: ignore_limit,
                })
                .await?;
            if !global.quiet {
                eprintln!("Device adoption initiated");
            }
            Ok(())
        }

        DevicesCommand::Remove { device } => {
            let id = util::resolve_device_id(controller, &device)?;
            if !util::confirm(&format!("Remove device {device}?"), global.yes)? {
                return Ok(());
            }
            controller.execute(CoreCommand::RemoveDevice { id }).await?;
            if !global.quiet {
                eprintln!("Device removed");
            }
            Ok(())
        }

        DevicesCommand::Restart { device } => {
            let id = util::resolve_device_id(controller, &device)?;
            controller
                .execute(CoreCommand::RestartDevice { id })
                .await?;
            if !global.quiet {
                eprintln!("Device restart initiated");
            }
            Ok(())
        }

        DevicesCommand::Locate { device, on } => {
            let mac = util::resolve_device_mac(controller, &device)?;
            controller
                .execute(CoreCommand::LocateDevice { mac, enable: on })
                .await?;
            if !global.quiet {
                let state = if on { "enabled" } else { "disabled" };
                eprintln!("Locate LED {state}");
            }
            Ok(())
        }

        DevicesCommand::PortCycle { device, port } => {
            let device_id = util::resolve_device_id(controller, &device)?;
            controller
                .execute(CoreCommand::PowerCyclePort {
                    device_id,
                    port_idx: port,
                })
                .await?;
            if !global.quiet {
                eprintln!("Port {port} power-cycled");
            }
            Ok(())
        }

        DevicesCommand::Stats { device } => {
            match find_device(controller, &device) {
                Some(device) => {
                    let out =
                        output::render_single(&global.output, &device, stats_detail, |device| {
                            device.id.to_string()
                        });
                    output::print_output(&out, global.quiet);
                }
                None => {
                    return Err(CliError::NotFound {
                        resource_type: "device".into(),
                        identifier: device,
                        list_command: "devices list".into(),
                    });
                }
            }
            Ok(())
        }

        DevicesCommand::Pending(list) => {
            let pending = util::apply_list_args(
                controller.list_pending_devices().await?,
                &list,
                util::matches_json_filter,
            );
            let out = output::render_list(
                &global.output,
                &pending,
                |value| pending_device_row(value, &painter),
                pending_device_identity,
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        DevicesCommand::Upgrade { device, url } => {
            let mac = util::resolve_device_mac(controller, &device)?;
            controller
                .execute(CoreCommand::UpgradeDevice {
                    mac,
                    firmware_url: url,
                })
                .await?;
            if !global.quiet {
                eprintln!("Firmware upgrade initiated");
            }
            Ok(())
        }

        DevicesCommand::Provision { device } => {
            let mac = util::resolve_device_mac(controller, &device)?;
            controller
                .execute(CoreCommand::ProvisionDevice { mac })
                .await?;
            if !global.quiet {
                eprintln!("Device re-provision initiated");
            }
            Ok(())
        }

        DevicesCommand::Speedtest => {
            controller.execute(CoreCommand::SpeedtestDevice).await?;
            if !global.quiet {
                eprintln!("Speed test initiated");
            }
            Ok(())
        }

        DevicesCommand::Tags(list) => {
            let tags = util::apply_list_args(
                controller.list_device_tags().await?,
                &list,
                |value, filter| util::matches_json_filter(value, filter),
            );
            let out = output::render_list(
                &global.output,
                &tags,
                |value| device_tag_row(value, &painter),
                device_tag_identity,
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }
    }
}
