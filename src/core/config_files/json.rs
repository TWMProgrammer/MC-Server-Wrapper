use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// Logic for .json files
pub fn flatten_json(prefix: &str, value: &JsonValue, props: &mut HashMap<String, String>) {
    match value {
        JsonValue::Object(obj) => {
            for (k, v) in obj {
                let new_prefix = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                flatten_json(&new_prefix, v, props);
            }
        }
        JsonValue::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                flatten_json(&new_prefix, v, props);
            }
        }
        JsonValue::String(s) => { props.insert(prefix.to_string(), s.clone()); }
        JsonValue::Number(n) => { props.insert(prefix.to_string(), n.to_string()); }
        JsonValue::Bool(b) => { props.insert(prefix.to_string(), b.to_string()); }
        JsonValue::Null => { props.insert(prefix.to_string(), "".to_string()); }
    }
}

pub fn unflatten_json(map: &mut serde_json::Map<String, JsonValue>, key: &str, value: String) {
    if let Some((prefix, suffix)) = key.split_once('.') {
        let entry = map.entry(prefix.to_string()).or_insert(JsonValue::Object(serde_json::Map::new()));
        if let JsonValue::Object(inner_map) = entry {
            unflatten_json(inner_map, suffix, value);
        }
    } else {
        let json_val = if value == "true" {
            JsonValue::Bool(true)
        } else if value == "false" {
            JsonValue::Bool(false)
        } else if let Ok(n) = value.parse::<i64>() {
            JsonValue::Number(n.into())
        } else if let Ok(f) = value.parse::<f64>() {
            if let Some(n) = serde_json::Number::from_f64(f) {
                JsonValue::Number(n)
            } else {
                JsonValue::String(value)
            }
        } else {
            JsonValue::String(value)
        };
        map.insert(key.to_string(), json_val);
    }
}
