use super::*;

/// Parse a numeric field from a JSON object, tolerating both string and number encodings.
fn parse_f64_field(parent: Option<&serde_json::Value>, key: &str) -> Option<f64> {
    parent.and_then(|value| value.get(key)).and_then(|value| {
        value
            .as_str()
            .and_then(|value| value.parse().ok())
            .or_else(|| value.as_f64())
    })
}

/// Apply a `device:sync` WebSocket message to the DataStore.
///
/// Extracts CPU, memory, load averages, and uplink bandwidth from the
/// raw Legacy API device JSON. Merges stats into the existing device
/// (looked up by MAC) without clobbering Integration API fields.
#[allow(clippy::cast_precision_loss)]
pub(super) fn apply_device_sync(store: &DataStore, data: &serde_json::Value) {
    let Some(mac_str) = data.get("mac").and_then(serde_json::Value::as_str) else {
        return;
    };
    let mac = MacAddress::new(mac_str);
    let Some(existing) = store.device_by_mac(&mac) else {
        return;
    };

    let sys = data.get("sys_stats");
    let cpu = sys
        .and_then(|value| value.get("cpu"))
        .and_then(|value| value.as_str().or_else(|| value.as_f64().map(|_| "")))
        .and_then(|value| {
            if value.is_empty() {
                None
            } else {
                value.parse::<f64>().ok()
            }
        })
        .or_else(|| {
            sys.and_then(|value| value.get("cpu"))
                .and_then(serde_json::Value::as_f64)
        });
    #[allow(clippy::as_conversions, clippy::cast_precision_loss)]
    let mem_pct = match (
        sys.and_then(|value| value.get("mem_used"))
            .and_then(serde_json::Value::as_i64),
        sys.and_then(|value| value.get("mem_total"))
            .and_then(serde_json::Value::as_i64),
    ) {
        (Some(used), Some(total)) if total > 0 => Some((used as f64 / total as f64) * 100.0),
        _ => None,
    };
    let load_averages: [Option<f64>; 3] =
        ["loadavg_1", "loadavg_5", "loadavg_15"].map(|key| parse_f64_field(sys, key));

    let uplink = data.get("uplink");
    let tx_bps = uplink
        .and_then(|value| value.get("tx_bytes-r").or_else(|| value.get("tx_bytes_r")))
        .and_then(serde_json::Value::as_u64)
        .or_else(|| data.get("tx_bytes-r").and_then(serde_json::Value::as_u64));
    let rx_bps = uplink
        .and_then(|value| value.get("rx_bytes-r").or_else(|| value.get("rx_bytes_r")))
        .and_then(serde_json::Value::as_u64)
        .or_else(|| data.get("rx_bytes-r").and_then(serde_json::Value::as_u64));

    let bandwidth = match (tx_bps, rx_bps) {
        (Some(tx), Some(rx)) if tx > 0 || rx > 0 => Some(crate::model::common::Bandwidth {
            tx_bytes_per_sec: tx,
            rx_bytes_per_sec: rx,
        }),
        _ => existing.stats.uplink_bandwidth,
    };

    let uptime = data
        .get("_uptime")
        .or_else(|| data.get("uptime"))
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| value.try_into().ok())
        .or(existing.stats.uptime_secs);

    let mut device = (*existing).clone();
    device.stats.uplink_bandwidth = bandwidth;
    if let Some(cpu) = cpu {
        device.stats.cpu_utilization_pct = Some(cpu);
    }
    if let Some(mem_pct) = mem_pct {
        device.stats.memory_utilization_pct = Some(mem_pct);
    }
    if let Some(load) = load_averages[0] {
        device.stats.load_average_1m = Some(load);
    }
    if let Some(load) = load_averages[1] {
        device.stats.load_average_5m = Some(load);
    }
    if let Some(load) = load_averages[2] {
        device.stats.load_average_15m = Some(load);
    }
    device.stats.uptime_secs = uptime;

    if let Some(num_sta) = data.get("num_sta").and_then(serde_json::Value::as_u64) {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        {
            device.client_count = Some(num_sta as u32);
        }
    }

    if let Some(object) = data.as_object()
        && let Some(wan_ipv6) = parse_legacy_device_wan_ipv6(object)
    {
        device.wan_ipv6 = Some(wan_ipv6);
    }

    let key = mac.as_str().to_owned();
    let id = device.id.clone();
    store.devices.upsert(key, id, device);
}

/// Process commands from the mpsc channel, routing each to the
/// appropriate Legacy API call.
pub(super) async fn command_processor_task(
    controller: Controller,
    mut rx: mpsc::Receiver<CommandEnvelope>,
) {
    let cancel = controller.inner.cancel_child.lock().await.clone();

    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => break,
            envelope = rx.recv() => {
                let Some(envelope) = envelope else { break };
                let result = super::commands::route_command(&controller, envelope.command).await;
                let _ = envelope.response_tx.send(result);
            }
        }
    }
}
