//! Shared helpers for command handlers.

use std::path::Path;

use unifly_api::{Controller, EntityId, MacAddress};

use crate::cli::args::ListArgs;
use crate::cli::error::CliError;

/// Resolve a device identifier (UUID or MAC) to an EntityId via snapshot lookup.
pub fn resolve_device_id(controller: &Controller, identifier: &str) -> Result<EntityId, CliError> {
    let snap = controller.devices_snapshot();
    for device in snap.iter() {
        if device.id.to_string() == identifier || device.mac.to_string() == identifier {
            return Ok(device.id.clone());
        }
    }
    Err(CliError::NotFound {
        resource_type: "device".into(),
        identifier: identifier.into(),
        list_command: "devices list".into(),
    })
}

/// Resolve a device identifier to a MacAddress via snapshot lookup.
#[allow(clippy::unnecessary_wraps)]
pub fn resolve_device_mac(
    controller: &Controller,
    identifier: &str,
) -> Result<MacAddress, CliError> {
    let snap = controller.devices_snapshot();
    for device in snap.iter() {
        if device.id.to_string() == identifier || device.mac.to_string() == identifier {
            return Ok(device.mac.clone());
        }
    }
    // If not in snapshot, treat the identifier itself as a MAC
    Ok(MacAddress::new(identifier))
}

/// Resolve a client identifier (UUID or MAC) to an EntityId via snapshot lookup.
#[allow(dead_code)]
pub fn resolve_client_id(controller: &Controller, identifier: &str) -> Result<EntityId, CliError> {
    let snap = controller.clients_snapshot();
    for client in snap.iter() {
        if client.id.to_string() == identifier || client.mac.to_string() == identifier {
            return Ok(client.id.clone());
        }
    }
    Err(CliError::NotFound {
        resource_type: "client".into(),
        identifier: identifier.into(),
        list_command: "clients list".into(),
    })
}

/// Prompt for confirmation, auto-approving if `--yes` was passed.
pub fn confirm(message: &str, yes_flag: bool) -> Result<bool, CliError> {
    if yes_flag {
        return Ok(true);
    }
    let confirmed = dialoguer::Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()
        .map_err(|e| CliError::Io(std::io::Error::other(e)))?;
    Ok(confirmed)
}

/// Read and parse a JSON file for `--from-file` flags.
pub fn read_json_file(path: &Path) -> Result<serde_json::Value, CliError> {
    let contents = std::fs::read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|e| CliError::Validation {
        field: "from-file".into(),
        reason: format!("invalid JSON: {e}"),
    })
}

/// Apply list flags (`--limit`, `--offset`, `--all`, `--filter`) to an iterator.
pub fn apply_list_args<T>(
    items: impl IntoIterator<Item = T>,
    list: &ListArgs,
    matches_filter: impl Fn(&T, &str) -> bool,
) -> Vec<T> {
    let offset = usize::try_from(list.offset).unwrap_or(usize::MAX);
    let limit = usize::try_from(list.limit).unwrap_or(usize::MAX);
    let filter = list
        .filter
        .as_deref()
        .map(str::trim)
        .filter(|f| !f.is_empty());

    let filtered = items.into_iter().filter(|item| match filter {
        Some(expr) => matches_filter(item, expr),
        None => true,
    });

    if list.all {
        filtered.skip(offset).collect()
    } else {
        filtered.skip(offset).take(limit).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FilterOp {
    Eq,
    Contains,
    In,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilterExpr {
    path: Vec<String>,
    op: FilterOp,
    values: Vec<String>,
}

/// Parse and evaluate a tiny local filter grammar:
/// `field.eq('value')`, `field.contains('value')`, or `field.in('a','b')`.
pub fn matches_json_filter<T: serde::Serialize>(item: &T, filter: &str) -> bool {
    let expr = match parse_filter_expr(filter) {
        Some(expr) => expr,
        None => return false,
    };

    let value = match serde_json::to_value(item) {
        Ok(value) => value,
        Err(_) => return false,
    };

    let mut matches = Vec::new();
    collect_path_values(&value, &expr.path, &mut matches);
    matches
        .into_iter()
        .any(|candidate| matches_filter_value(candidate, &expr.op, &expr.values))
}

fn parse_filter_expr(filter: &str) -> Option<FilterExpr> {
    let filter = filter.trim();
    if filter.is_empty() {
        return None;
    }

    let open_paren = filter.find('(')?;
    let close_paren = filter.rfind(')')?;
    if close_paren != filter.len().saturating_sub(1) || close_paren <= open_paren {
        return None;
    }

    let head = filter[..open_paren].trim();
    let args = filter[open_paren + 1..close_paren].trim();
    let dot = head.rfind('.')?;
    let path = head[..dot]
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if path.is_empty() {
        return None;
    }

    let op = match head[dot + 1..].trim().to_ascii_lowercase().as_str() {
        "eq" => FilterOp::Eq,
        "contains" => FilterOp::Contains,
        "in" => FilterOp::In,
        _ => return None,
    };

    let values = parse_quoted_values(args)?;
    match op {
        FilterOp::Eq | FilterOp::Contains if values.len() != 1 => return None,
        FilterOp::In if values.is_empty() => return None,
        _ => {}
    }

    Some(FilterExpr { path, op, values })
}

fn parse_quoted_values(input: &str) -> Option<Vec<String>> {
    let mut values = Vec::new();
    let mut chars = input.chars().peekable();

    loop {
        while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        let quote = match chars.next()? {
            '\'' => '\'',
            '"' => '"',
            _ => return None,
        };

        let mut value = String::new();
        let mut escaped = false;
        let mut closed = false;
        while let Some(ch) = chars.next() {
            if escaped {
                let decoded = match ch {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    other => other,
                };
                value.push(decoded);
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                c if c == quote => {
                    closed = true;
                    break;
                }
                other => value.push(other),
            }
        }

        if !closed || escaped {
            return None;
        }

        values.push(value);

        while matches!(chars.peek(), Some(ch) if ch.is_whitespace()) {
            chars.next();
        }

        match chars.peek() {
            Some(',') => {
                chars.next();
            }
            None => break,
            _ => return None,
        }
    }

    Some(values)
}

fn collect_path_values<'a>(
    value: &'a serde_json::Value,
    path: &[String],
    out: &mut Vec<&'a serde_json::Value>,
) {
    if path.is_empty() {
        out.push(value);
        return;
    }

    match value {
        serde_json::Value::Object(map) => {
            if let Some(next) = map.get(&path[0]) {
                collect_path_values(next, &path[1..], out);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_path_values(item, path, out);
            }
        }
        _ => {}
    }
}

fn scalar_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Number(number) => Some(number.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        serde_json::Value::Null => Some("null".into()),
        _ => None,
    }
}

fn matches_filter_value(value: &serde_json::Value, op: &FilterOp, candidates: &[String]) -> bool {
    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .any(|item| matches_filter_value(item, op, candidates)),
        serde_json::Value::Object(_) => false,
        scalar => {
            let Some(text) = scalar_text(scalar) else {
                return false;
            };
            let normalized = text.to_ascii_lowercase();

            match op {
                FilterOp::Eq => candidates
                    .iter()
                    .any(|candidate| normalized == candidate.to_ascii_lowercase()),
                FilterOp::Contains => candidates
                    .iter()
                    .any(|candidate| normalized.contains(&candidate.to_ascii_lowercase())),
                FilterOp::In => candidates
                    .iter()
                    .any(|candidate| normalized == candidate.to_ascii_lowercase()),
            }
        }
    }
}

pub async fn ensure_integration_access(
    controller: &Controller,
    operation: &str,
) -> Result<(), CliError> {
    if controller.has_integration_access().await {
        return Ok(());
    }

    Err(CliError::Unsupported {
        operation: operation.into(),
        required: "Integration API".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::{apply_list_args, matches_json_filter};
    use crate::cli::args::ListArgs;

    #[test]
    fn apply_list_args_respects_offset_limit() {
        let args = ListArgs {
            limit: 2,
            offset: 1,
            all: false,
            filter: None,
        };
        let rows = vec![1, 2, 3, 4];
        let sliced = apply_list_args(rows, &args, |_, _| true);
        assert_eq!(sliced, vec![2, 3]);
    }

    #[test]
    fn apply_list_args_supports_local_expression_filters() {
        let args = ListArgs {
            limit: 25,
            offset: 0,
            all: false,
            filter: Some("name.eq('beta')".into()),
        };
        let rows = vec![
            serde_json::json!({"name":"alpha"}),
            serde_json::json!({"name":"beta"}),
        ];
        let filtered = apply_list_args(rows, &args, |item, filter| {
            matches_json_filter(item, filter)
        });
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0]["name"], "beta");
    }

    #[test]
    fn matches_json_filter_supports_contains_and_in() {
        let item = serde_json::json!({
            "name": "Alpha Beta",
            "state": "online",
            "tags": ["primary", "uplink"],
            "nested": {
                "ssid": "NovaNet"
            }
        });

        assert!(matches_json_filter(&item, "name.contains('beta')"));
        assert!(matches_json_filter(&item, "state.in('offline', 'ONLINE')"));
        assert!(matches_json_filter(&item, "nested.ssid.eq('novanet')"));
        assert!(!matches_json_filter(&item, "name.eq('gamma')"));
        assert!(!matches_json_filter(&item, "Alpha Beta"));
        assert!(!matches_json_filter(&item, "name.starts_with('alpha')"));
    }
}
