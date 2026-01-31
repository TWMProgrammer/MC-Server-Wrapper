use std::collections::HashMap;
use serde_yaml::Value as YamlValue;

/// Logic for .yml and .yaml files
pub fn flatten_yaml(prefix: &str, value: &YamlValue, props: &mut HashMap<String, String>) {
    match value {
        YamlValue::Mapping(map) => {
            if map.is_empty() && !prefix.is_empty() {
                props.insert(prefix.to_string(), "{}".to_string());
            } else {
                for (k, v) in map {
                    if let Some(key_str) = k.as_str() {
                        let new_prefix = if prefix.is_empty() { key_str.to_string() } else { format!("{}.{}", prefix, key_str) };
                        flatten_yaml(&new_prefix, v, props);
                    }
                }
            }
        }
        YamlValue::Sequence(seq) => {
            if seq.is_empty() && !prefix.is_empty() {
                props.insert(prefix.to_string(), "[]".to_string());
            } else {
                for (i, v) in seq.iter().enumerate() {
                    let new_prefix = format!("{}[{}]", prefix, i);
                    flatten_yaml(&new_prefix, v, props);
                }
            }
        }
        YamlValue::String(s) => { props.insert(prefix.to_string(), s.clone()); }
        YamlValue::Number(n) => { props.insert(prefix.to_string(), n.to_string()); }
        YamlValue::Bool(b) => { props.insert(prefix.to_string(), b.to_string()); }
        YamlValue::Null => { props.insert(prefix.to_string(), "".to_string()); }
        _ => {}
    }
}

pub fn unflatten_yaml(map: &mut serde_yaml::Mapping, key: &str, value: String) {
    if let Some((prefix, suffix)) = key.split_once('.') {
        let entry = map.entry(YamlValue::String(prefix.to_string())).or_insert(YamlValue::Mapping(serde_yaml::Mapping::new()));
        if let YamlValue::Mapping(inner_map) = entry {
            unflatten_yaml(inner_map, suffix, value);
        }
    } else {
        // Try to parse as boolean or number if possible
        let yaml_val = if value == "true" {
            YamlValue::Bool(true)
        } else if value == "false" {
            YamlValue::Bool(false)
        } else if let Ok(n) = value.parse::<i64>() {
            YamlValue::Number(n.into())
        } else if let Ok(f) = value.parse::<f64>() {
            YamlValue::Number(f.into())
        } else {
            YamlValue::String(value)
        };
        map.insert(YamlValue::String(key.to_string()), yaml_val);
    }
}
