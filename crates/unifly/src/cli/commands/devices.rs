//! Device command handlers.

use std::sync::Arc;

use tabled::Tabled;
use unifly_api::{Command as CoreCommand, Controller, Device, MacAddress};

use crate::cli::args::{DevicesArgs, DevicesCommand, GlobalOpts};
use crate::cli::error::CliError;
use crate::cli::output;

use super::util;

// ── Table row ───────────────────────────────────────────────────────

#[derive(Tabled)]
struct DeviceRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Model")]
    model: String,
    #[tabled(rename = "Type")]
    dtype: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "IP")]
    ip: String,
    #[tabled(rename = "MAC")]
    mac: String,
}

#[derive(Tabled)]
struct PendingDeviceRow {
    #[tabled(rename = "IP")]
    ip: String,
    #[tabled(rename = "Model")]
    model: String,
    #[tabled(rename = "MAC")]
    mac: String,
    #[tabled(rename = "State")]
    state: String,
    #[tabled(rename = "Version")]
    firmware: String,
    #[tabled(rename = "Supported")]
    supported: String,
}

#[derive(Tabled)]
struct DeviceTagRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
}

fn device_row(d: &Arc<Device>, p: &output::Painter) -> DeviceRow {
    DeviceRow {
        id: p.id(&d.id.to_string()),
        name: p.name(&d.name.clone().unwrap_or_default()),
        model: p.muted(&d.model.clone().unwrap_or_default()),
        dtype: p.muted(&format!("{:?}", d.device_type)),
        state: p.state(&format!("{:?}", d.state)),
        ip: p.ip(&d.ip.map(|ip| ip.to_string()).unwrap_or_default()),
        mac: p.mac(&d.mac.to_string()),
    }
}

fn detail(d: &Arc<Device>) -> String {
    let mut lines = vec![
        format!("ID:       {}", d.id),
        format!("Name:     {}", d.name.as_deref().unwrap_or("-")),
        format!("MAC:      {}", d.mac),
        format!(
            "IP:       {}",
            d.ip.map_or_else(|| "-".into(), |ip| ip.to_string())
        ),
        format!("Model:    {}", d.model.as_deref().unwrap_or("-")),
        format!("Type:     {:?}", d.device_type),
        format!("State:    {:?}", d.state),
        format!("Firmware: {}", d.firmware_version.as_deref().unwrap_or("-")),
    ];
    if let Some(up) = d.stats.uptime_secs {
        lines.push(format!("Uptime:   {up}s"));
    }
    if let Some(cpu) = d.stats.cpu_utilization_pct {
        lines.push(format!("CPU:      {cpu:.1}%"));
    }
    if let Some(mem) = d.stats.memory_utilization_pct {
        lines.push(format!("Memory:   {mem:.1}%"));
    }
    lines.join("\n")
}

fn stats_detail(d: &Arc<Device>) -> String {
    [
        format!("ID:          {}", d.id),
        format!("Name:        {}", d.name.as_deref().unwrap_or("-")),
        format!("MAC:         {}", d.mac),
        format!(
            "Uptime:      {}",
            d.stats
                .uptime_secs
                .map_or_else(|| "-".into(), |v| format!("{v}s"))
        ),
        format!(
            "CPU:         {}",
            d.stats
                .cpu_utilization_pct
                .map_or_else(|| "-".into(), |v| format!("{v:.1}%"))
        ),
        format!(
            "Memory:      {}",
            d.stats
                .memory_utilization_pct
                .map_or_else(|| "-".into(), |v| format!("{v:.1}%"))
        ),
        format!(
            "Load Avg 1m: {}",
            d.stats
                .load_average_1m
                .map_or_else(|| "-".into(), |v| format!("{v:.2}"))
        ),
        format!(
            "Load Avg 5m: {}",
            d.stats
                .load_average_5m
                .map_or_else(|| "-".into(), |v| format!("{v:.2}"))
        ),
        format!(
            "Load Avg15m: {}",
            d.stats
                .load_average_15m
                .map_or_else(|| "-".into(), |v| format!("{v:.2}"))
        ),
    ]
    .join("\n")
}

fn pending_string<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
}

fn pending_device_row(v: &serde_json::Value, p: &output::Painter) -> PendingDeviceRow {
    PendingDeviceRow {
        ip: p.ip(pending_string(v, "ipAddress")),
        model: p.muted(pending_string(v, "model")),
        mac: p.mac(
            v.get("macAddress")
                .or_else(|| v.get("mac"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
        ),
        state: {
            let s = pending_string(v, "state");
            p.state(if s.is_empty() { "PENDING" } else { s })
        },
        firmware: p.muted(pending_string(v, "firmwareVersion")),
        supported: p.enabled(
            v.get("supported")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        ),
    }
}

fn pending_device_identity(v: &serde_json::Value) -> String {
    v.get("macAddress")
        .or_else(|| v.get("mac"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| v.get("ipAddress").and_then(serde_json::Value::as_str))
        .unwrap_or("")
        .to_owned()
}

// ── Handler ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
pub async fn handle(
    controller: &Controller,
    args: DevicesArgs,
    global: &GlobalOpts,
) -> Result<(), CliError> {
    let p = output::Painter::new(global);

    match args.command {
        DevicesCommand::List(list) => {
            let all = controller.devices_snapshot();
            let snap = util::apply_list_args(all.iter().cloned(), &list, |d, filter| {
                util::matches_json_filter(d, filter)
            });
            let out = output::render_list(
                &global.output,
                &snap,
                |d| device_row(d, &p),
                |d| d.id.to_string(),
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }

        DevicesCommand::Get { device } => {
            let snap = controller.devices_snapshot();
            let found = snap
                .iter()
                .find(|d| d.id.to_string() == device || d.mac.to_string() == device);
            match found {
                Some(d) => {
                    let out =
                        output::render_single(&global.output, d, detail, |d| d.id.to_string());
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
            let mac = MacAddress::new(&mac);
            controller
                .execute(CoreCommand::AdoptDevice {
                    mac,
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
            let snap = controller.devices_snapshot();
            let found = snap
                .iter()
                .find(|d| d.id.to_string() == device || d.mac.to_string() == device);
            match found {
                Some(d) => {
                    let out = output::render_single(&global.output, d, stats_detail, |d| {
                        d.id.to_string()
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
                |v| pending_device_row(v, &p),
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
            let tags =
                util::apply_list_args(controller.list_device_tags().await?, &list, |v, filter| {
                    util::matches_json_filter(v, filter)
                });
            let out = output::render_list(
                &global.output,
                &tags,
                |v| DeviceTagRow {
                    id: p.id(v
                        .get("id")
                        .or_else(|| v.get("_id"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")),
                    name: p.name(
                        v.get("name")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or(""),
                    ),
                },
                |v| {
                    v.get("id")
                        .or_else(|| v.get("_id"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_owned()
                },
            );
            output::print_output(&out, global.quiet);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{pending_device_identity, pending_device_row};
    use crate::cli::args::{ColorMode, OutputFormat};
    use crate::cli::output::Painter;

    fn plain_painter() -> Painter {
        Painter::new(&crate::cli::args::GlobalOpts {
            profile: None,
            controller: None,
            site: None,
            api_key: None,
            output: OutputFormat::Plain,
            color: ColorMode::Never,
            verbose: 0,
            quiet: false,
            yes: false,
            insecure: false,
            timeout: 30,
        })
    }

    #[test]
    fn pending_device_row_uses_actual_api_fields() {
        let row = pending_device_row(
            &serde_json::json!({
                "macAddress": "aa:bb:cc:dd:ee:ff",
                "ipAddress": "10.0.0.20",
                "model": "U7-Pro",
                "state": "DISCOVERED",
                "firmwareVersion": "1.2.3",
                "supported": true
            }),
            &plain_painter(),
        );

        assert_eq!(row.ip, "10.0.0.20");
        assert_eq!(row.model, "U7-Pro");
        assert_eq!(row.mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(row.state, "DISCOVERED");
        assert_eq!(row.firmware, "1.2.3");
        assert_eq!(row.supported, "yes");
    }

    #[test]
    fn pending_device_identity_prefers_mac_address() {
        let identity = pending_device_identity(&serde_json::json!({
            "macAddress": "aa:bb:cc:dd:ee:ff",
            "ipAddress": "10.0.0.20"
        }));
        assert_eq!(identity, "aa:bb:cc:dd:ee:ff");
    }
}
