use std::net::Ipv4Addr;

use crate::core_error::CoreError;
use crate::model::WifiSecurityMode;

pub(in super::super) fn parse_ipv4_cidr(cidr: &str) -> Result<(Ipv4Addr, u8), CoreError> {
    let (host, prefix) = cidr
        .split_once('/')
        .ok_or_else(|| CoreError::ValidationFailed {
            message: format!("invalid ipv4 host/prefix value '{cidr}'"),
        })?;
    let host_ip = host
        .parse::<Ipv4Addr>()
        .map_err(|_| CoreError::ValidationFailed {
            message: format!("invalid IPv4 host address '{host}'"),
        })?;
    let prefix_len = prefix
        .parse::<u8>()
        .map_err(|_| CoreError::ValidationFailed {
            message: format!("invalid IPv4 prefix length '{prefix}'"),
        })?;
    if prefix_len > 32 {
        return Err(CoreError::ValidationFailed {
            message: format!("IPv4 prefix length must be <= 32, got {prefix_len}"),
        });
    }
    Ok((host_ip, prefix_len))
}

fn wifi_security_mode_name(mode: WifiSecurityMode) -> &'static str {
    match mode {
        WifiSecurityMode::Open => "OPEN",
        WifiSecurityMode::Wpa2Personal => "WPA2_PERSONAL",
        WifiSecurityMode::Wpa3Personal => "WPA3_PERSONAL",
        WifiSecurityMode::Wpa2Wpa3Personal => "WPA2_WPA3_PERSONAL",
        WifiSecurityMode::Wpa2Enterprise => "WPA2_ENTERPRISE",
        WifiSecurityMode::Wpa3Enterprise => "WPA3_ENTERPRISE",
        WifiSecurityMode::Wpa2Wpa3Enterprise => "WPA2_WPA3_ENTERPRISE",
    }
}

fn wifi_payload_name(name: &str, ssid: &str) -> String {
    if name.is_empty() {
        ssid.to_owned()
    } else {
        name.to_owned()
    }
}

fn wifi_frequency_values(frequencies: &[f32]) -> Vec<serde_json::Value> {
    frequencies
        .iter()
        .map(|frequency| {
            // Parse through the string representation to avoid f32→f64
            // precision artifacts (e.g. 2.4f32 → 2.4000000953674316f64).
            let s = format!("{frequency}");
            serde_json::Number::from_f64(s.parse::<f64>().unwrap_or(f64::from(*frequency)))
                .map_or(serde_json::Value::Null, serde_json::Value::Number)
        })
        .collect()
}

fn ensure_wifi_payload_defaults(
    body: &mut serde_json::Map<String, serde_json::Value>,
    broadcast_type: &str,
) {
    body.entry("clientIsolationEnabled")
        .or_insert(serde_json::Value::Bool(false));
    body.entry("multicastToUnicastConversionEnabled")
        .or_insert(serde_json::Value::Bool(false));
    body.entry("hideName")
        .or_insert(serde_json::Value::Bool(false));
    body.entry("uapsdEnabled")
        .or_insert(serde_json::Value::Bool(true));

    if broadcast_type == "STANDARD" {
        body.entry("broadcastingFrequenciesGHz")
            .or_insert_with(|| serde_json::Value::Array(wifi_frequency_values(&[2.4, 5.0])));
        body.entry("mloEnabled")
            .or_insert(serde_json::Value::Bool(false));
        body.entry("bandSteeringEnabled")
            .or_insert(serde_json::Value::Bool(false));
        body.entry("arpProxyEnabled")
            .or_insert(serde_json::Value::Bool(false));
        body.entry("bssTransitionEnabled")
            .or_insert(serde_json::Value::Bool(false));
        body.entry("advertiseDeviceName")
            .or_insert(serde_json::Value::Bool(false));
    }
}

pub(in super::super) fn build_create_wifi_broadcast_payload(
    req: &crate::command::CreateWifiBroadcastRequest,
) -> crate::integration_types::WifiBroadcastCreateUpdate {
    let broadcast_type = req
        .broadcast_type
        .clone()
        .unwrap_or_else(|| "STANDARD".into());

    let mut body = serde_json::Map::new();
    let mut security_configuration = serde_json::Map::new();
    security_configuration.insert(
        "mode".into(),
        serde_json::Value::String(wifi_security_mode_name(req.security_mode).into()),
    );
    if let Some(passphrase) = req.passphrase.clone() {
        security_configuration.insert("passphrase".into(), serde_json::Value::String(passphrase));
    }
    body.insert(
        "securityConfiguration".into(),
        serde_json::Value::Object(security_configuration),
    );

    if let Some(network_id) = &req.network_id {
        body.insert(
            "network".into(),
            serde_json::json!({ "id": network_id.to_string() }),
        );
    }
    body.insert("hideName".into(), serde_json::Value::Bool(req.hide_ssid));
    if req.band_steering {
        body.insert("bandSteeringEnabled".into(), serde_json::Value::Bool(true));
    }
    if req.fast_roaming {
        body.insert("bssTransitionEnabled".into(), serde_json::Value::Bool(true));
    }
    if let Some(frequencies) = req.frequencies_ghz.as_ref() {
        body.insert(
            "broadcastingFrequenciesGHz".into(),
            serde_json::Value::Array(wifi_frequency_values(frequencies)),
        );
    }
    ensure_wifi_payload_defaults(&mut body, &broadcast_type);

    crate::integration_types::WifiBroadcastCreateUpdate {
        name: wifi_payload_name(&req.name, &req.ssid),
        broadcast_type,
        enabled: req.enabled,
        body,
    }
}

pub(in super::super) fn build_update_wifi_broadcast_payload(
    existing: &crate::integration_types::WifiBroadcastDetailsResponse,
    update: &crate::command::UpdateWifiBroadcastRequest,
) -> crate::integration_types::WifiBroadcastCreateUpdate {
    let mut body: serde_json::Map<String, serde_json::Value> =
        existing.extra.clone().into_iter().collect();

    body.remove("ssid");
    body.remove("hideSsid");
    body.remove("bandSteering");
    body.remove("fastRoaming");
    body.remove("frequencies");

    if let Some(network) = existing.network.clone() {
        body.insert("network".into(), network);
    }
    if let Some(filter) = existing.broadcasting_device_filter.clone() {
        body.insert("broadcastingDeviceFilter".into(), filter);
    }
    if let Some(hidden) = update.hide_ssid {
        body.insert("hideName".into(), serde_json::Value::Bool(hidden));
    }

    let mut security_cfg = existing
        .security_configuration
        .as_object()
        .cloned()
        .unwrap_or_default();
    if let Some(mode) = update.security_mode {
        security_cfg.insert(
            "mode".into(),
            serde_json::Value::String(wifi_security_mode_name(mode).into()),
        );
    }
    if let Some(passphrase) = update.passphrase.clone() {
        security_cfg.insert("passphrase".into(), serde_json::Value::String(passphrase));
    }
    body.insert(
        "securityConfiguration".into(),
        serde_json::Value::Object(security_cfg),
    );
    ensure_wifi_payload_defaults(&mut body, &existing.broadcast_type);

    crate::integration_types::WifiBroadcastCreateUpdate {
        name: update
            .name
            .clone()
            .or_else(|| update.ssid.clone())
            .unwrap_or_else(|| existing.name.clone()),
        broadcast_type: existing.broadcast_type.clone(),
        enabled: update.enabled.unwrap_or(existing.enabled),
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_create_wifi_broadcast_payload, parse_ipv4_cidr};
    use crate::command::CreateWifiBroadcastRequest;
    use crate::model::WifiSecurityMode;
    use serde_json::json;

    #[test]
    fn parse_ipv4_cidr_accepts_valid_input() {
        let (host, prefix) = parse_ipv4_cidr("192.168.10.1/24").expect("valid CIDR");
        assert_eq!(host.to_string(), "192.168.10.1");
        assert_eq!(prefix, 24);
    }

    #[test]
    fn parse_ipv4_cidr_rejects_invalid_prefix() {
        assert!(parse_ipv4_cidr("192.168.10.1/40").is_err());
    }

    #[test]
    fn parse_ipv4_cidr_rejects_missing_prefix() {
        assert!(parse_ipv4_cidr("192.168.10.1").is_err());
    }

    #[test]
    fn wifi_create_payload_uses_integration_field_names() {
        let payload = build_create_wifi_broadcast_payload(&CreateWifiBroadcastRequest {
            name: "Main".into(),
            ssid: "Main".into(),
            security_mode: WifiSecurityMode::Wpa2Personal,
            passphrase: Some("supersecret".into()),
            enabled: true,
            network_id: None,
            hide_ssid: true,
            broadcast_type: Some("STANDARD".into()),
            frequencies_ghz: Some(vec![2.4, 5.0]),
            band_steering: true,
            fast_roaming: true,
        });

        assert_eq!(payload.name, "Main");
        assert!(payload.body.get("ssid").is_none());
        assert_eq!(payload.body.get("hideName"), Some(&json!(true)));
        let frequencies = payload
            .body
            .get("broadcastingFrequenciesGHz")
            .and_then(serde_json::Value::as_array)
            .expect("frequencies array");
        assert_eq!(frequencies.len(), 2);
        assert_eq!(frequencies[1], json!(5.0));
        assert_eq!(payload.body.get("bandSteeringEnabled"), Some(&json!(true)));
        assert_eq!(payload.body.get("bssTransitionEnabled"), Some(&json!(true)));
    }

    #[test]
    fn wifi_frequency_values_avoid_f32_precision_artifacts() {
        let payload = build_create_wifi_broadcast_payload(&CreateWifiBroadcastRequest {
            name: "Test".into(),
            ssid: "Test".into(),
            security_mode: WifiSecurityMode::Wpa3Personal,
            passphrase: Some("secret".into()),
            enabled: true,
            network_id: None,
            hide_ssid: false,
            broadcast_type: Some("STANDARD".into()),
            frequencies_ghz: Some(vec![2.4, 5.0, 6.0]),
            band_steering: false,
            fast_roaming: false,
        });

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("2.4"), "expected 2.4 not f64 artifact");
        assert!(!json.contains("2.400000"), "f32→f64 precision artifact");
    }
}
