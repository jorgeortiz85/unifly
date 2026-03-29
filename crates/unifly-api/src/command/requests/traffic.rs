use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTrafficMatchingListRequest {
    pub name: String,
    #[serde(default = "default_traffic_list_type")]
    pub list_type: String,
    pub entries: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "items")]
    pub raw_items: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateTrafficMatchingListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(alias = "items")]
    pub raw_items: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_traffic_list_type() -> String {
    "IPV4".into()
}

#[cfg(test)]
mod tests {
    use super::CreateTrafficMatchingListRequest;

    #[test]
    fn create_traffic_matching_list_defaults_type_and_reads_items_alias() {
        let request: CreateTrafficMatchingListRequest = serde_json::from_value(serde_json::json!({
            "name": "RFC1918",
            "entries": ["10.0.0.0/8"],
            "items": [{"type": "subnet", "value": "10.0.0.0/8"}]
        }))
        .unwrap();

        assert_eq!(request.list_type, "IPV4");
        assert_eq!(request.raw_items.as_ref().map(std::vec::Vec::len), Some(1));
    }
}
