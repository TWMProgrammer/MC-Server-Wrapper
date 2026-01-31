use std::collections::HashMap;
use toml::Value as TomlValue;

/// Logic for .toml files
pub fn flatten_toml(prefix: &str, value: &TomlValue, props: &mut HashMap<String, String>) {
    match value {
        TomlValue::Table(table) => {
            for (k, v) in table {
                let new_prefix = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                flatten_toml(&new_prefix, v, props);
            }
        }
        TomlValue::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                flatten_toml(&new_prefix, v, props);
            }
        }
        TomlValue::String(s) => { props.insert(prefix.to_string(), s.clone()); }
        TomlValue::Integer(i) => { props.insert(prefix.to_string(), i.to_string()); }
        TomlValue::Float(f) => { props.insert(prefix.to_string(), f.to_string()); }
        TomlValue::Boolean(b) => { props.insert(prefix.to_string(), b.to_string()); }
        TomlValue::Datetime(d) => { props.insert(prefix.to_string(), d.to_string()); }
    }
}

pub fn unflatten_toml(map: &mut toml::map::Map<String, TomlValue>, key: &str, value: String) {
    if let Some((prefix, suffix)) = key.split_once('.') {
        let entry = map.entry(prefix.to_string()).or_insert(TomlValue::Table(toml::map::Map::new()));
        if let TomlValue::Table(inner_map) = entry {
            unflatten_toml(inner_map, suffix, value);
        }
    } else {
        let toml_val = if value == "true" {
            TomlValue::Boolean(true)
        } else if value == "false" {
            TomlValue::Boolean(false)
        } else if let Ok(n) = value.parse::<i64>() {
            TomlValue::Integer(n)
        } else if let Ok(f) = value.parse::<f64>() {
            TomlValue::Float(f)
        } else {
            TomlValue::String(value)
        };
        map.insert(key.to_string(), toml_val);
    }
}
