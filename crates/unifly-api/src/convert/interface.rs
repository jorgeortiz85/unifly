use serde_json::Value;

use crate::model::device::{PoeInfo, Port, PortConnector, PortState, Radio};

fn parse_port_state(raw: &str) -> PortState {
    match raw {
        "UP" | "up" => PortState::Up,
        "DOWN" | "down" => PortState::Down,
        _ => PortState::Unknown,
    }
}

fn parse_port_connector(raw: &str) -> Option<PortConnector> {
    match raw {
        "RJ45" | "rj45" => Some(PortConnector::Rj45),
        "SFP" | "sfp" => Some(PortConnector::Sfp),
        "SFPPLUS" | "SFP+" | "sfp+" => Some(PortConnector::SfpPlus),
        "SFP28" | "sfp28" => Some(PortConnector::Sfp28),
        "QSFP28" | "qsfp28" => Some(PortConnector::Qsfp28),
        _ => None,
    }
}

pub(crate) fn parse_integration_ports(interfaces: &Value) -> Vec<Port> {
    let Some(ports) = interfaces.get("ports").and_then(Value::as_array) else {
        return Vec::new();
    };
    ports
        .iter()
        .filter_map(|p| {
            let idx = p.get("idx").and_then(Value::as_u64)?;
            #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
            Some(Port {
                index: idx as u32,
                name: p.get("name").and_then(Value::as_str).map(String::from),
                state: p
                    .get("state")
                    .and_then(Value::as_str)
                    .map_or(PortState::Unknown, parse_port_state),
                speed_mbps: p.get("speedMbps").and_then(Value::as_u64).map(|v| v as u32),
                max_speed_mbps: p
                    .get("maxSpeedMbps")
                    .and_then(Value::as_u64)
                    .map(|v| v as u32),
                connector: p
                    .get("connector")
                    .and_then(Value::as_str)
                    .and_then(parse_port_connector),
                poe: p.get("poe").map(|poe| PoeInfo {
                    standard: poe
                        .get("standard")
                        .and_then(Value::as_str)
                        .map(String::from),
                    enabled: poe.get("enabled").and_then(Value::as_bool).unwrap_or(false),
                    state: poe
                        .get("state")
                        .and_then(Value::as_str)
                        .map_or(PortState::Unknown, parse_port_state),
                }),
            })
        })
        .collect()
}

pub(crate) fn parse_integration_radios(interfaces: &Value) -> Vec<Radio> {
    let Some(radios) = interfaces.get("radios").and_then(Value::as_array) else {
        return Vec::new();
    };
    radios
        .iter()
        .filter_map(|r| {
            #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
            let freq = r.get("frequencyGHz").and_then(Value::as_f64)? as f32;
            #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
            Some(Radio {
                frequency_ghz: freq,
                channel: r.get("channel").and_then(Value::as_u64).map(|v| v as u32),
                channel_width_mhz: r
                    .get("channelWidthMHz")
                    .and_then(Value::as_u64)
                    .map(|v| v as u32),
                wlan_standard: r
                    .get("wlanStandard")
                    .and_then(Value::as_str)
                    .map(String::from),
                tx_retries_pct: r.get("txRetriesPct").and_then(Value::as_f64),
                channel_utilization_pct: None,
            })
        })
        .collect()
}

pub(crate) fn enrich_radios_from_stats(radios: &mut [Radio], stats_interfaces: &Value) {
    let Some(stats_radios) = stats_interfaces.get("radios").and_then(Value::as_array) else {
        return;
    };
    for sr in stats_radios {
        #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
        let Some(freq) = sr
            .get("frequencyGHz")
            .and_then(Value::as_f64)
            .map(|f| f as f32)
        else {
            continue;
        };
        let retries = sr.get("txRetriesPct").and_then(Value::as_f64);
        if let Some(radio) = radios
            .iter_mut()
            .find(|r| (r.frequency_ghz - freq).abs() < 0.1)
            .filter(|r| r.tx_retries_pct.is_none())
        {
            radio.tx_retries_pct = retries;
        }
    }
}

pub(crate) fn parse_session_ports(extra: &serde_json::Map<String, Value>) -> Vec<Port> {
    let Some(ports) = extra.get("port_table").and_then(Value::as_array) else {
        return Vec::new();
    };
    ports
        .iter()
        .filter_map(|p| {
            let idx = p.get("port_idx").and_then(Value::as_u64)?;
            let up = p.get("up").and_then(Value::as_bool).unwrap_or(false);
            #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
            Some(Port {
                index: idx as u32,
                name: p.get("name").and_then(Value::as_str).map(String::from),
                state: if up { PortState::Up } else { PortState::Down },
                speed_mbps: p.get("speed").and_then(Value::as_u64).map(|v| v as u32),
                max_speed_mbps: None,
                connector: p
                    .get("media")
                    .and_then(Value::as_str)
                    .and_then(|m| match m {
                        "GE" | "FE" => Some(PortConnector::Rj45),
                        "SFP" => Some(PortConnector::Sfp),
                        "SFP+" => Some(PortConnector::SfpPlus),
                        _ => None,
                    }),
                poe: if p.get("port_poe").and_then(Value::as_bool).unwrap_or(false) {
                    Some(PoeInfo {
                        standard: p.get("poe_caps").and_then(Value::as_u64).map(|caps| {
                            match caps {
                                7 => "802.3bt",
                                3 => "802.3at",
                                _ => "802.3af",
                            }
                            .to_owned()
                        }),
                        enabled: p
                            .get("poe_enable")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        state: if p.get("poe_good").and_then(Value::as_bool).unwrap_or(false) {
                            PortState::Up
                        } else {
                            PortState::Down
                        },
                    })
                } else {
                    None
                },
            })
        })
        .collect()
}

fn session_radio_freq(band: &str) -> Option<f32> {
    match band {
        "ng" => Some(2.4),
        "na" => Some(5.0),
        "6e" => Some(6.0),
        _ => None,
    }
}

pub(crate) fn parse_session_radios(extra: &serde_json::Map<String, Value>) -> Vec<Radio> {
    let Some(radios) = extra.get("radio_table").and_then(Value::as_array) else {
        return Vec::new();
    };
    let stats = extra
        .get("radio_table_stats")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    radios
        .iter()
        .filter_map(|r| {
            let band = r.get("radio").and_then(Value::as_str)?;
            let freq = session_radio_freq(band)?;
            let stat = stats.iter().find(|s| {
                s.get("radio")
                    .and_then(Value::as_str)
                    .is_some_and(|b| b == band)
            });
            #[allow(clippy::cast_possible_truncation, clippy::as_conversions)]
            Some(Radio {
                frequency_ghz: freq,
                channel: r
                    .get("channel")
                    .or_else(|| stat.and_then(|s| s.get("channel")))
                    .and_then(Value::as_u64)
                    .map(|v| v as u32),
                channel_width_mhz: r
                    .get("ht")
                    .and_then(Value::as_str)
                    .and_then(|ht| ht.parse::<u32>().ok()),
                wlan_standard: None,
                tx_retries_pct: None,
                channel_utilization_pct: stat
                    .and_then(|s| s.get("cu_total"))
                    .and_then(Value::as_f64),
            })
        })
        .collect()
}
