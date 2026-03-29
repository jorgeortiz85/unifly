use std::net::Ipv4Addr;

use crate::command::requests::TrafficFilterSpec;
use crate::core_error::CoreError;
use crate::model::{DnsPolicyType, EntityId, WifiSecurityMode};

pub(super) fn parse_ipv4_cidr(cidr: &str) -> Result<(Ipv4Addr, u8), CoreError> {
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

pub(super) fn build_acl_filter_value(
    zone_id: &EntityId,
    port: Option<&str>,
    protocol: Option<&str>,
) -> serde_json::Value {
    let mut filter = serde_json::Map::new();
    filter.insert(
        "zoneId".into(),
        serde_json::Value::String(zone_id.to_string()),
    );
    if let Some(port) = port {
        filter.insert("port".into(), serde_json::Value::String(port.to_owned()));
    }
    if let Some(protocol) = protocol {
        filter.insert(
            "protocol".into(),
            serde_json::Value::String(protocol.to_owned()),
        );
    }
    serde_json::Value::Object(filter)
}

pub(super) fn merge_acl_filter_value(
    existing: Option<serde_json::Value>,
    zone_id: Option<&EntityId>,
    port: Option<&str>,
    protocol: Option<&str>,
) -> Option<serde_json::Value> {
    let mut filter = match existing {
        Some(serde_json::Value::Object(filter)) => filter,
        Some(_) | None => serde_json::Map::new(),
    };

    if let Some(zone_id) = zone_id {
        filter.insert(
            "zoneId".into(),
            serde_json::Value::String(zone_id.to_string()),
        );
    }
    if let Some(port) = port {
        filter.insert("port".into(), serde_json::Value::String(port.to_owned()));
    }
    if let Some(protocol) = protocol {
        filter.insert(
            "protocol".into(),
            serde_json::Value::String(protocol.to_owned()),
        );
    }

    (!filter.is_empty()).then_some(serde_json::Value::Object(filter))
}

pub(super) fn build_endpoint_json(
    zone_id: &str,
    filter: Option<&TrafficFilterSpec>,
) -> serde_json::Value {
    let mut obj = serde_json::json!({ "zoneId": zone_id });

    if let Some(spec) = filter {
        let traffic_filter = match spec {
            TrafficFilterSpec::Network {
                network_ids,
                match_opposite,
            } => {
                serde_json::json!({
                    "type": "NETWORK",
                    "networkFilter": {
                        "networkIds": network_ids,
                        "matchOpposite": match_opposite,
                    }
                })
            }
            TrafficFilterSpec::IpAddress {
                addresses,
                match_opposite,
            } => {
                let items: Vec<serde_json::Value> = addresses
                    .iter()
                    .map(|addr| {
                        if addr.contains('/') {
                            serde_json::json!({ "type": "SUBNET", "value": addr })
                        } else if addr.contains('-') {
                            let parts: Vec<&str> = addr.splitn(2, '-').collect();
                            serde_json::json!({ "type": "RANGE", "start": parts[0], "stop": parts.get(1).unwrap_or(&"") })
                        } else {
                            serde_json::json!({ "type": "IP_ADDRESS", "value": addr })
                        }
                    })
                    .collect();
                serde_json::json!({
                    "type": "IP_ADDRESSES",
                    "ipAddressFilter": {
                        "type": "IP_ADDRESSES",
                        "items": items,
                        "matchOpposite": match_opposite,
                    }
                })
            }
            TrafficFilterSpec::Port {
                ports,
                match_opposite,
            } => {
                let items: Vec<serde_json::Value> = ports
                    .iter()
                    .map(|p| {
                        if p.contains('-') {
                            let parts: Vec<&str> = p.splitn(2, '-').collect();
                            serde_json::json!({ "type": "PORT_RANGE", "startPort": parts[0], "endPort": parts.get(1).unwrap_or(&"") })
                        } else {
                            serde_json::json!({ "type": "PORT_NUMBER", "value": p })
                        }
                    })
                    .collect();
                serde_json::json!({
                    "type": "PORT",
                    "portFilter": {
                        "type": "PORTS",
                        "items": items,
                        "matchOpposite": match_opposite,
                    }
                })
            }
        };
        obj.as_object_mut()
            .expect("json! produces object")
            .insert("trafficFilter".into(), traffic_filter);
    }

    obj
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

pub(super) fn dns_policy_type_name(policy_type: DnsPolicyType) -> &'static str {
    match policy_type {
        DnsPolicyType::ARecord => "A",
        DnsPolicyType::AaaaRecord => "AAAA",
        DnsPolicyType::CnameRecord => "CNAME",
        DnsPolicyType::MxRecord => "MX",
        DnsPolicyType::TxtRecord => "TXT",
        DnsPolicyType::SrvRecord => "SRV",
        DnsPolicyType::ForwardDomain => "FORWARD_DOMAIN",
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
        .map(|frequency| serde_json::Value::from(f64::from(*frequency)))
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

pub(super) fn build_create_wifi_broadcast_payload(
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

pub(super) fn build_update_wifi_broadcast_payload(
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

fn dns_policy_type_from_name(policy_type: &str) -> DnsPolicyType {
    match policy_type {
        "A" => DnsPolicyType::ARecord,
        "AAAA" => DnsPolicyType::AaaaRecord,
        "CNAME" => DnsPolicyType::CnameRecord,
        "MX" => DnsPolicyType::MxRecord,
        "TXT" => DnsPolicyType::TxtRecord,
        "SRV" => DnsPolicyType::SrvRecord,
        _ => DnsPolicyType::ForwardDomain,
    }
}

fn validation_failed(message: impl Into<String>) -> CoreError {
    CoreError::ValidationFailed {
        message: message.into(),
    }
}

fn dns_domain_value(
    domain: Option<&str>,
    domains: Option<&[String]>,
    fallback: Option<&str>,
) -> Option<String> {
    domain
        .map(str::to_owned)
        .or_else(|| domains.and_then(|values| values.first().cloned()))
        .or_else(|| {
            fallback
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
        })
}

fn insert_string_field(
    fields: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<String>,
) {
    if let Some(value) = value {
        fields.insert(key.into(), serde_json::Value::String(value));
    }
}

fn insert_u16_field(
    fields: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<u16>,
) {
    if let Some(value) = value {
        fields.insert(
            key.into(),
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
    }
}

fn insert_u32_field(
    fields: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<u32>,
) {
    if let Some(value) = value {
        fields.insert(
            key.into(),
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
    }
}

fn ensure_dns_required_string(
    fields: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    policy_type: DnsPolicyType,
) -> Result<(), CoreError> {
    if fields
        .get(key)
        .and_then(serde_json::Value::as_str)
        .is_some()
    {
        Ok(())
    } else {
        Err(validation_failed(format!(
            "{policy_type:?} DNS policy requires `{key}`"
        )))
    }
}

fn ensure_dns_required_number(
    fields: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    policy_type: DnsPolicyType,
) -> Result<(), CoreError> {
    if fields
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .is_some()
    {
        Ok(())
    } else {
        Err(validation_failed(format!(
            "{policy_type:?} DNS policy requires `{key}`"
        )))
    }
}

fn validate_dns_policy_fields(
    policy_type: DnsPolicyType,
    fields: &serde_json::Map<String, serde_json::Value>,
) -> Result<(), CoreError> {
    ensure_dns_required_string(fields, "domain", policy_type)?;

    match policy_type {
        DnsPolicyType::ARecord => {
            ensure_dns_required_string(fields, "ipv4Address", policy_type)?;
            ensure_dns_required_number(fields, "ttlSeconds", policy_type)?;
        }
        DnsPolicyType::AaaaRecord => {
            ensure_dns_required_string(fields, "ipv6Address", policy_type)?;
            ensure_dns_required_number(fields, "ttlSeconds", policy_type)?;
        }
        DnsPolicyType::CnameRecord => {
            ensure_dns_required_string(fields, "targetDomain", policy_type)?;
            ensure_dns_required_number(fields, "ttlSeconds", policy_type)?;
        }
        DnsPolicyType::MxRecord => {
            ensure_dns_required_string(fields, "mailServerDomain", policy_type)?;
            ensure_dns_required_number(fields, "priority", policy_type)?;
        }
        DnsPolicyType::TxtRecord => {
            ensure_dns_required_string(fields, "text", policy_type)?;
        }
        DnsPolicyType::SrvRecord => {
            for key in ["serverDomain", "service", "protocol"] {
                ensure_dns_required_string(fields, key, policy_type)?;
            }
            for key in ["port", "priority", "weight"] {
                ensure_dns_required_number(fields, key, policy_type)?;
            }
        }
        DnsPolicyType::ForwardDomain => {
            ensure_dns_required_string(fields, "ipAddress", policy_type)?;
        }
    }

    Ok(())
}

pub(super) fn build_create_dns_policy_fields(
    req: &crate::command::CreateDnsPolicyRequest,
) -> Result<serde_json::Map<String, serde_json::Value>, CoreError> {
    let mut fields = serde_json::Map::new();
    let domain = dns_domain_value(
        req.domain.as_deref(),
        req.domains.as_deref(),
        Some(req.name.as_str()),
    )
    .ok_or_else(|| validation_failed("DNS policy requires `domain`"))?;
    fields.insert("domain".into(), serde_json::Value::String(domain));

    match req.policy_type {
        DnsPolicyType::ARecord => {
            insert_string_field(
                &mut fields,
                "ipv4Address",
                req.ipv4_address.clone().or_else(|| req.value.clone()),
            );
            insert_u32_field(&mut fields, "ttlSeconds", req.ttl_seconds);
        }
        DnsPolicyType::AaaaRecord => {
            insert_string_field(
                &mut fields,
                "ipv6Address",
                req.ipv6_address.clone().or_else(|| req.value.clone()),
            );
            insert_u32_field(&mut fields, "ttlSeconds", req.ttl_seconds);
        }
        DnsPolicyType::CnameRecord => {
            insert_string_field(
                &mut fields,
                "targetDomain",
                req.target_domain.clone().or_else(|| req.value.clone()),
            );
            insert_u32_field(&mut fields, "ttlSeconds", req.ttl_seconds);
        }
        DnsPolicyType::MxRecord => {
            insert_string_field(
                &mut fields,
                "mailServerDomain",
                req.mail_server_domain.clone().or_else(|| req.value.clone()),
            );
            insert_u16_field(&mut fields, "priority", req.priority);
        }
        DnsPolicyType::TxtRecord => {
            insert_string_field(
                &mut fields,
                "text",
                req.text.clone().or_else(|| req.value.clone()),
            );
        }
        DnsPolicyType::SrvRecord => {
            insert_string_field(
                &mut fields,
                "serverDomain",
                req.server_domain.clone().or_else(|| req.value.clone()),
            );
            insert_string_field(&mut fields, "service", req.service.clone());
            insert_string_field(&mut fields, "protocol", req.protocol.clone());
            insert_u16_field(&mut fields, "port", req.port);
            insert_u16_field(&mut fields, "priority", req.priority);
            insert_u16_field(&mut fields, "weight", req.weight);
        }
        DnsPolicyType::ForwardDomain => {
            insert_string_field(
                &mut fields,
                "ipAddress",
                req.ip_address
                    .clone()
                    .or_else(|| req.upstream.clone())
                    .or_else(|| req.value.clone()),
            );
        }
    }

    validate_dns_policy_fields(req.policy_type, &fields)?;
    Ok(fields)
}

pub(super) fn build_update_dns_policy_fields(
    existing: &crate::integration_types::DnsPolicyResponse,
    update: &crate::command::UpdateDnsPolicyRequest,
) -> Result<serde_json::Map<String, serde_json::Value>, CoreError> {
    let policy_type = dns_policy_type_from_name(&existing.policy_type);
    let mut fields: serde_json::Map<String, serde_json::Value> =
        existing.extra.clone().into_iter().collect();

    if let Some(domain) = dns_domain_value(
        update.domain.as_deref(),
        update.domains.as_deref(),
        existing.domain.as_deref(),
    ) {
        fields.insert("domain".into(), serde_json::Value::String(domain));
    }

    match policy_type {
        DnsPolicyType::ARecord => {
            insert_string_field(
                &mut fields,
                "ipv4Address",
                update.ipv4_address.clone().or_else(|| update.value.clone()),
            );
            insert_u32_field(&mut fields, "ttlSeconds", update.ttl_seconds);
        }
        DnsPolicyType::AaaaRecord => {
            insert_string_field(
                &mut fields,
                "ipv6Address",
                update.ipv6_address.clone().or_else(|| update.value.clone()),
            );
            insert_u32_field(&mut fields, "ttlSeconds", update.ttl_seconds);
        }
        DnsPolicyType::CnameRecord => {
            insert_string_field(
                &mut fields,
                "targetDomain",
                update
                    .target_domain
                    .clone()
                    .or_else(|| update.value.clone()),
            );
            insert_u32_field(&mut fields, "ttlSeconds", update.ttl_seconds);
        }
        DnsPolicyType::MxRecord => {
            insert_string_field(
                &mut fields,
                "mailServerDomain",
                update
                    .mail_server_domain
                    .clone()
                    .or_else(|| update.value.clone()),
            );
            insert_u16_field(&mut fields, "priority", update.priority);
        }
        DnsPolicyType::TxtRecord => {
            insert_string_field(
                &mut fields,
                "text",
                update.text.clone().or_else(|| update.value.clone()),
            );
        }
        DnsPolicyType::SrvRecord => {
            insert_string_field(
                &mut fields,
                "serverDomain",
                update
                    .server_domain
                    .clone()
                    .or_else(|| update.value.clone()),
            );
            insert_string_field(&mut fields, "service", update.service.clone());
            insert_string_field(&mut fields, "protocol", update.protocol.clone());
            insert_u16_field(&mut fields, "port", update.port);
            insert_u16_field(&mut fields, "priority", update.priority);
            insert_u16_field(&mut fields, "weight", update.weight);
        }
        DnsPolicyType::ForwardDomain => {
            insert_string_field(
                &mut fields,
                "ipAddress",
                update
                    .ip_address
                    .clone()
                    .or_else(|| update.upstream.clone())
                    .or_else(|| update.value.clone()),
            );
        }
    }

    validate_dns_policy_fields(policy_type, &fields)?;
    Ok(fields)
}

pub(super) fn traffic_matching_list_items(
    entries: &[String],
    raw_items: Option<&[serde_json::Value]>,
) -> Vec<serde_json::Value> {
    raw_items.map_or_else(
        || {
            entries
                .iter()
                .cloned()
                .map(serde_json::Value::String)
                .collect()
        },
        <[serde_json::Value]>::to_vec,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_create_dns_policy_fields, build_create_wifi_broadcast_payload, parse_ipv4_cidr,
        traffic_matching_list_items,
    };
    use crate::command::{CreateDnsPolicyRequest, CreateWifiBroadcastRequest};
    use crate::model::{DnsPolicyType, WifiSecurityMode};
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
    fn dns_create_fields_use_type_specific_schema_keys() {
        let fields = build_create_dns_policy_fields(&CreateDnsPolicyRequest {
            name: "example.com".into(),
            policy_type: DnsPolicyType::ARecord,
            enabled: true,
            domain: Some("example.com".into()),
            domains: None,
            upstream: None,
            value: Some("192.168.1.10".into()),
            ttl_seconds: Some(600),
            priority: None,
            ipv4_address: None,
            ipv6_address: None,
            target_domain: None,
            mail_server_domain: None,
            text: None,
            ip_address: None,
            server_domain: None,
            service: None,
            protocol: None,
            port: None,
            weight: None,
        })
        .expect("valid DNS fields");

        assert_eq!(fields.get("domain"), Some(&json!("example.com")));
        assert_eq!(fields.get("ipv4Address"), Some(&json!("192.168.1.10")));
        assert_eq!(fields.get("ttlSeconds"), Some(&json!(600)));
        assert!(fields.get("value").is_none());
        assert!(fields.get("ttl").is_none());
    }

    #[test]
    fn traffic_matching_list_items_prefer_raw_payloads() {
        let raw_items = [json!({"type": "PORT_NUMBER", "value": 443})];
        let items = traffic_matching_list_items(&["80".into()], Some(&raw_items));
        assert_eq!(items, vec![json!({"type": "PORT_NUMBER", "value": 443})]);
    }
}
