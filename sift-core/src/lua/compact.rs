//! JSON compaction utilities — truncate long strings, summarize large arrays, limit depth/keys.

use serde_json;

/// Compact a JSON value: truncate long strings, summarize large arrays, limit depth/keys.
pub fn compact_json(
    val: &serde_json::Value,
    max_string_len: usize,
    max_array_items: usize,
    max_depth: usize,
    max_keys: usize,
) -> String {
    let compacted = compact_value(val, max_string_len, max_array_items, max_depth, max_keys, 0);
    serde_json::to_string(&compacted).unwrap_or_default()
}

fn compact_value(
    val: &serde_json::Value,
    max_string_len: usize,
    max_array_items: usize,
    max_depth: usize,
    max_keys: usize,
    depth: usize,
) -> serde_json::Value {
    if depth > max_depth {
        return serde_json::Value::String("...".to_string());
    }
    match val {
        serde_json::Value::String(s) => {
            if s.len() > max_string_len {
                let truncated: String = s.chars().take(max_string_len).collect();
                serde_json::Value::String(format!("{truncated}..."))
            } else {
                serde_json::Value::String(s.clone())
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.len() > max_array_items {
                let mut items: Vec<serde_json::Value> = arr[..max_array_items]
                    .iter()
                    .map(|v| compact_value(v, max_string_len, max_array_items, max_depth, max_keys, depth + 1))
                    .collect();
                let remaining = arr.len() - max_array_items;
                items.push(serde_json::Value::String(format!("... +{remaining} more")));
                serde_json::Value::Array(items)
            } else {
                serde_json::Value::Array(
                    arr.iter()
                        .map(|v| compact_value(v, max_string_len, max_array_items, max_depth, max_keys, depth + 1))
                        .collect(),
                )
            }
        }
        serde_json::Value::Object(obj) => {
            let entries: Vec<(String, serde_json::Value)> = obj
                .iter()
                .take(max_keys)
                .map(|(k, v)| (k.clone(), compact_value(v, max_string_len, max_array_items, max_depth, max_keys, depth + 1)))
                .collect();
            let mut map = serde_json::Map::new();
            for (k, v) in entries {
                map.insert(k, v);
            }
            if obj.len() > max_keys {
                map.insert("...".to_string(), serde_json::Value::String(format!("+{} more keys", obj.len() - max_keys)));
            }
            serde_json::Value::Object(map)
        }
        other => other.clone(),
    }
}
