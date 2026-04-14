use crate::command::requests::TrafficFilterSpec;

fn build_port_filter_json(ports: &[String]) -> serde_json::Value {
    let items: Vec<serde_json::Value> = ports
        .iter()
        .map(|p| {
            if p.contains('-') {
                let range: Vec<&str> = p.splitn(2, '-').collect();
                let start: u16 = range[0].parse().unwrap_or(0);
                let end: u16 = range.get(1).unwrap_or(&"0").parse().unwrap_or(0);
                serde_json::json!({ "type": "PORT_NUMBER_RANGE", "startPort": start, "endPort": end })
            } else {
                let port: u16 = p.parse().unwrap_or(0);
                serde_json::json!({ "type": "PORT_NUMBER", "value": port })
            }
        })
        .collect();
    serde_json::json!({
        "type": "PORTS",
        "items": items,
        "matchOpposite": false,
    })
}

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
                ports,
            } => {
                let mut v = serde_json::json!({
                    "type": "NETWORK",
                    "networkFilter": {
                        "networkIds": network_ids,
                        "matchOpposite": match_opposite,
                    }
                });
                if let Some(ports) = ports {
                    v.as_object_mut()
                        .expect("json! produces object")
                        .insert("portFilter".into(), build_port_filter_json(ports));
                }
                v
            }
            TrafficFilterSpec::IpAddress {
                addresses,
                match_opposite,
                ports,
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
                let mut v = serde_json::json!({
                    "type": "IP_ADDRESS",
                    "ipAddressFilter": {
                        "type": "IP_ADDRESSES",
                        "items": items,
                        "matchOpposite": match_opposite,
                    }
                });
                if let Some(ports) = ports {
                    v.as_object_mut()
                        .expect("json! produces object")
                        .insert("portFilter".into(), build_port_filter_json(ports));
                }
                v
            }
            TrafficFilterSpec::Port {
                ports,
                match_opposite,
            } => {
                let mut pf = build_port_filter_json(ports);
                pf.as_object_mut()
                    .expect("json! produces object")
                    .insert("matchOpposite".into(), (*match_opposite).into());
                serde_json::json!({
                    "type": "PORT",
                    "portFilter": pf,
                })
            }
            TrafficFilterSpec::PortMatchingList {
                list_id,
                match_opposite,
            } => {
                serde_json::json!({
                    "type": "PORT",
                    "portFilter": {
                        "type": "TRAFFIC_MATCHING_LIST",
                        "trafficMatchingListId": list_id,
                        "matchOpposite": match_opposite,
                    }
                })
            }
            TrafficFilterSpec::IpMatchingList {
                list_id,
                match_opposite,
            } => {
                serde_json::json!({
                    "type": "IP_ADDRESS",
                    "ipAddressFilter": {
                        "type": "TRAFFIC_MATCHING_LIST",
                        "trafficMatchingListId": list_id,
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
    use super::{build_endpoint_json, traffic_matching_list_items};
    use crate::command::requests::TrafficFilterSpec;
    use serde_json::json;

    #[test]
    fn traffic_matching_list_items_prefer_raw_payloads() {
        let raw_items = [json!({"type": "PORT_NUMBER", "value": 443})];
        let items = traffic_matching_list_items(&["80".into()], Some(&raw_items));
        assert_eq!(items, vec![json!({"type": "PORT_NUMBER", "value": 443})]);
    }

    #[test]
    fn build_endpoint_json_ip_with_port_nests_port_filter() {
        let spec = TrafficFilterSpec::IpAddress {
            addresses: vec!["10.0.40.10".into()],
            match_opposite: false,
            ports: Some(vec!["80".into()]),
        };
        let result = build_endpoint_json("zone-uuid", Some(&spec));
        let tf = result.get("trafficFilter").expect("trafficFilter present");
        assert_eq!(tf.get("type").and_then(|v| v.as_str()), Some("IP_ADDRESS"));
        // ipAddressFilter should be present
        let ip_filter = tf.get("ipAddressFilter").expect("ipAddressFilter present");
        assert_eq!(
            ip_filter
                .get("items")
                .and_then(|v| v.as_array())
                .map(Vec::len),
            Some(1)
        );
        // portFilter should be nested alongside ipAddressFilter
        let port_filter = tf.get("portFilter").expect("portFilter present");
        assert_eq!(
            port_filter
                .get("items")
                .and_then(|v| v.as_array())
                .map(Vec::len),
            Some(1)
        );
        let port_item = &port_filter["items"][0];
        assert_eq!(port_item["type"].as_str(), Some("PORT_NUMBER"));
        assert_eq!(port_item["value"].as_u64(), Some(80));
    }

    #[test]
    fn build_endpoint_json_port_matching_list() {
        let spec = TrafficFilterSpec::PortMatchingList {
            list_id: "24740a56-9cb9-4890-a5ac-589d30914a55".into(),
            match_opposite: false,
        };
        let result = build_endpoint_json("zone-uuid", Some(&spec));
        let tf = result.get("trafficFilter").expect("trafficFilter present");
        assert_eq!(tf.get("type").and_then(|v| v.as_str()), Some("PORT"));
        let pf = tf.get("portFilter").expect("portFilter present");
        assert_eq!(
            pf.get("type").and_then(|v| v.as_str()),
            Some("TRAFFIC_MATCHING_LIST")
        );
        assert_eq!(
            pf.get("trafficMatchingListId").and_then(|v| v.as_str()),
            Some("24740a56-9cb9-4890-a5ac-589d30914a55")
        );
        assert_eq!(
            pf.get("matchOpposite").and_then(serde_json::Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn build_endpoint_json_ip_matching_list() {
        let spec = TrafficFilterSpec::IpMatchingList {
            list_id: "b777b27c-410c-4b40-8489-a61bf1a536d4".into(),
            match_opposite: false,
        };
        let result = build_endpoint_json("zone-uuid", Some(&spec));
        let tf = result.get("trafficFilter").expect("trafficFilter present");
        assert_eq!(tf.get("type").and_then(|v| v.as_str()), Some("IP_ADDRESS"));
        let ipf = tf.get("ipAddressFilter").expect("ipAddressFilter present");
        assert_eq!(
            ipf.get("type").and_then(|v| v.as_str()),
            Some("TRAFFIC_MATCHING_LIST")
        );
        assert_eq!(
            ipf.get("trafficMatchingListId").and_then(|v| v.as_str()),
            Some("b777b27c-410c-4b40-8489-a61bf1a536d4")
        );
    }

    #[test]
    fn build_endpoint_json_ip_without_port_omits_port_filter() {
        let spec = TrafficFilterSpec::IpAddress {
            addresses: vec!["10.0.0.1".into()],
            match_opposite: false,
            ports: None,
        };
        let result = build_endpoint_json("zone-uuid", Some(&spec));
        let tf = result.get("trafficFilter").expect("trafficFilter present");
        assert!(tf.get("portFilter").is_none());
    }
}
