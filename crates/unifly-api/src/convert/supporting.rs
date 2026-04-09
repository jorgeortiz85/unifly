use serde_json::Value;

use crate::integration_types;
use crate::model::common::DataSource;
use crate::model::entity_id::EntityId;
use crate::model::hotspot::Voucher;
use crate::model::supporting::TrafficMatchingList;

use super::helpers::parse_iso;

fn traffic_matching_item_to_string(item: &Value) -> Option<String> {
    match item {
        Value::String(value) => Some(value.clone()),
        Value::Object(map) => {
            if let Some(value) = map
                .get("value")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| {
                    map.get("value")
                        .and_then(Value::as_i64)
                        .map(|value| value.to_string())
                })
            {
                return Some(value);
            }

            let start = map.get("start").or_else(|| map.get("startPort"));
            let stop = map.get("stop").or_else(|| map.get("endPort"));
            match (start, stop) {
                (Some(start), Some(stop)) => {
                    let start = start
                        .as_str()
                        .map(str::to_owned)
                        .or_else(|| start.as_i64().map(|value| value.to_string()));
                    let stop = stop
                        .as_str()
                        .map(str::to_owned)
                        .or_else(|| stop.as_i64().map(|value| value.to_string()));
                    match (start, stop) {
                        (Some(start), Some(stop)) => Some(format!("{start}-{stop}")),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

impl From<integration_types::TrafficMatchingListResponse> for TrafficMatchingList {
    fn from(t: integration_types::TrafficMatchingListResponse) -> Self {
        let items = t
            .extra
            .get("items")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(traffic_matching_item_to_string)
                    .collect()
            })
            .unwrap_or_default();

        TrafficMatchingList {
            id: EntityId::Uuid(t.id),
            name: t.name,
            list_type: t.list_type,
            items,
            origin: None,
        }
    }
}

impl From<integration_types::VoucherResponse> for Voucher {
    fn from(v: integration_types::VoucherResponse) -> Self {
        #[allow(
            clippy::as_conversions,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        Voucher {
            id: EntityId::Uuid(v.id),
            code: v.code,
            name: Some(v.name),
            created_at: parse_iso(&v.created_at),
            activated_at: v.activated_at.as_deref().and_then(parse_iso),
            expires_at: v.expires_at.as_deref().and_then(parse_iso),
            expired: v.expired,
            time_limit_minutes: Some(v.time_limit_minutes as u32),
            data_usage_limit_mb: v.data_usage_limit_m_bytes.map(|b| b as u64),
            authorized_guest_limit: v.authorized_guest_limit.map(|l| l as u32),
            authorized_guest_count: Some(v.authorized_guest_count as u32),
            rx_rate_limit_kbps: v.rx_rate_limit_kbps.map(|r| r as u64),
            tx_rate_limit_kbps: v.tx_rate_limit_kbps.map(|r| r as u64),
            source: DataSource::IntegrationApi,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use serde_json::json;

    #[test]
    fn integration_traffic_matching_list_formats_structured_items() {
        let response = integration_types::TrafficMatchingListResponse {
            id: uuid::Uuid::nil(),
            name: "Ports".into(),
            list_type: "PORT".into(),
            extra: HashMap::from([(
                "items".into(),
                json!([
                    {"type": "PORT_NUMBER", "value": 443},
                    {"type": "PORT_RANGE", "start": 1000, "stop": 2000}
                ]),
            )]),
        };

        let list = TrafficMatchingList::from(response);
        assert_eq!(list.items, vec!["443".to_owned(), "1000-2000".to_owned()]);
    }
}
