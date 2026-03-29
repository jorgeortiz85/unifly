use crate::cli::args::ListArgs;

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
        .filter(|expr| !expr.is_empty());

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
    let Some(expr) = parse_filter_expr(filter) else {
        return false;
    };

    let Ok(value) = serde_json::to_value(item) else {
        return false;
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
        for ch in chars.by_ref() {
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
