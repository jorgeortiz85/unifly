use crate::model::EntityId;

pub(in super::super) fn build_acl_filter_value(
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

pub(in super::super) fn merge_acl_filter_value(
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
