use crate::command::requests::TrafficFilterSpec;

pub(in super::super) fn build_endpoint_json(
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
                            let start: u16 = parts[0].parse().unwrap_or(0);
                            let end: u16 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
                            serde_json::json!({ "type": "PORT_RANGE", "startPort": start, "endPort": end })
                        } else {
                            let port: u16 = p.parse().unwrap_or(0);
                            serde_json::json!({ "type": "PORT_NUMBER", "value": port })
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

pub(in super::super) fn traffic_matching_list_items(
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
    use super::traffic_matching_list_items;
    use serde_json::json;

    #[test]
    fn traffic_matching_list_items_prefer_raw_payloads() {
        let raw_items = [json!({"type": "PORT_NUMBER", "value": 443})];
        let items = traffic_matching_list_items(&["80".into()], Some(&raw_items));
        assert_eq!(items, vec![json!({"type": "PORT_NUMBER", "value": 443})]);
    }
}
