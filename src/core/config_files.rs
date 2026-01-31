use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};
use tokio::fs;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use toml::Value as TomlValue;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub path: String, // Relative to instance root
    pub format: ConfigFormat,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ConfigFormat {
    Properties,
    Yaml,
    Toml,
    Json,
}

pub async fn list_available_configs(instance_path: &Path, _mod_loader: Option<&str>) -> Vec<ConfigFile> {
    let mut configs = vec![
        ConfigFile {
            name: "server.properties".to_string(),
            path: "server.properties".to_string(),
            format: ConfigFormat::Properties,
        },
    ];

    // Check for common files in root
    let root_files = [
        ("bukkit.yml", ConfigFormat::Yaml),
        ("spigot.yml", ConfigFormat::Yaml),
        ("paper.yml", ConfigFormat::Yaml),
        ("purpur.yml", ConfigFormat::Yaml),
        ("pufferfish.yml", ConfigFormat::Yaml),
        ("commands.yml", ConfigFormat::Yaml),
        ("help.yml", ConfigFormat::Yaml),
        ("permissions.yml", ConfigFormat::Yaml),
        ("fabric-loader.json", ConfigFormat::Json),
    ];

    for (file, format) in root_files {
        if instance_path.join(file).exists() {
            // Avoid duplicates if already added
            if !configs.iter().any(|c| c.path == file) {
                configs.push(ConfigFile {
                    name: file.to_string(),
                    path: file.to_string(),
                    format: format.clone(),
                });
            }
        }
    }

    // Check config directory
    let config_dir = instance_path.join("config");
    if config_dir.exists() {
        if let Ok(mut entries) = fs::read_dir(config_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                        let rel_path = format!("config/{}", file_name);

                        // Support TOML (Forge/NeoForge)
                        if ext_str == "toml" {
                            configs.push(ConfigFile {
                                name: file_name,
                                path: rel_path,
                                format: ConfigFormat::Toml,
                            });
                        }
                        // Support YAML (Paper 1.19+, etc.)
                        else if ext_str == "yml" || ext_str == "yaml" {
                            configs.push(ConfigFile {
                                name: file_name,
                                path: rel_path,
                                format: ConfigFormat::Yaml,
                            });
                        }
                        // Support JSON
                        else if ext_str == "json" {
                            configs.push(ConfigFile {
                                name: file_name,
                                path: rel_path,
                                format: ConfigFormat::Json,
                            });
                        }
                    }
                }
            }
        }
    }

    configs
}

pub async fn read_config_file(instance_path: &Path, rel_path: &str, format: ConfigFormat) -> Result<HashMap<String, String>> {
    let full_path = instance_path.join(rel_path);
    if !full_path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&full_path).await
        .context(format!("Failed to read config file: {}", rel_path))?;

    match format {
        ConfigFormat::Properties => {
            let mut props = HashMap::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    props.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            Ok(props)
        }
        ConfigFormat::Yaml => {
            let yaml: YamlValue = serde_yaml::from_str(&content)?;
            let mut props = HashMap::new();
            flatten_yaml("", &yaml, &mut props);
            Ok(props)
        }
        ConfigFormat::Toml => {
            let toml: TomlValue = toml::from_str(&content)?;
            let mut props = HashMap::new();
            flatten_toml("", &toml, &mut props);
            Ok(props)
        }
        ConfigFormat::Json => {
            let json: JsonValue = serde_json::from_str(&content)?;
            let mut props = HashMap::new();
            flatten_json("", &json, &mut props);
            Ok(props)
        }
    }
}

pub async fn read_config_value(instance_path: &Path, rel_path: &str, format: ConfigFormat) -> Result<JsonValue> {
    let full_path = instance_path.join(rel_path);
    if !full_path.exists() {
        return Ok(JsonValue::Null);
    }

    let content = fs::read_to_string(&full_path).await
        .context(format!("Failed to read config file: {}", rel_path))?;

    match format {
        ConfigFormat::Properties => {
            let mut map = serde_json::Map::new();
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    map.insert(key.trim().to_string(), JsonValue::String(value.trim().to_string()));
                }
            }
            Ok(JsonValue::Object(map))
        }
        ConfigFormat::Yaml => {
            let yaml: YamlValue = serde_yaml::from_str(&content)?;
            // Convert YamlValue to JsonValue
            let json_str = serde_json::to_string(&yaml)?;
            let json: JsonValue = serde_json::from_str(&json_str)?;
            Ok(json)
        }
        ConfigFormat::Toml => {
            let toml: TomlValue = toml::from_str(&content)?;
            let json_str = serde_json::to_string(&toml)?;
            let json: JsonValue = serde_json::from_str(&json_str)?;
            Ok(json)
        }
        ConfigFormat::Json => {
            let json: JsonValue = serde_json::from_str(&content)?;
            Ok(json)
        }
    }
}

pub async fn save_config_value(instance_path: &Path, rel_path: &str, format: ConfigFormat, value: JsonValue) -> Result<()> {
    let full_path = instance_path.join(rel_path);
    
    let content = match format {
        ConfigFormat::Properties => {
            let mut content = String::new();
            content.push_str("# Generated by MC Server Wrapper\n");
            if let JsonValue::Object(map) = value {
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                for key in keys {
                    if let Some(val) = map.get(key) {
                        let val_str = match val {
                            JsonValue::String(s) => s.clone(),
                            _ => val.to_string(),
                        };
                        content.push_str(&format!("{}={}\n", key, val_str));
                    }
                }
            }
            content
        }
        ConfigFormat::Yaml => {
            let yaml: YamlValue = serde_json::from_value(value)?;
            serde_yaml::to_string(&yaml)?
        }
        ConfigFormat::Toml => {
            let toml: TomlValue = serde_json::from_value(value)?;
            toml::to_string_pretty(&toml)?
        }
        ConfigFormat::Json => {
            serde_json::to_string_pretty(&value)?
        }
    };

    fs::write(&full_path, content).await
        .context(format!("Failed to write config file: {}", rel_path))?;
    
    Ok(())
}

pub async fn save_config_file(instance_path: &Path, rel_path: &str, format: ConfigFormat, properties: HashMap<String, String>) -> Result<()> {
    let full_path = instance_path.join(rel_path);
    
    let content = match format {
        ConfigFormat::Properties => {
            let mut content = String::new();
            content.push_str("# Generated by MC Server Wrapper\n");
            let mut keys: Vec<_> = properties.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(value) = properties.get(key) {
                    content.push_str(&format!("{}={}\n", key, value));
                }
            }
            content
        }
        ConfigFormat::Yaml => {
            // To save YAML properly while preserving structure, we need to unflatten it
            let mut yaml_map = serde_yaml::Mapping::new();
            for (key, value) in properties {
                unflatten_yaml(&mut yaml_map, &key, value);
            }
            serde_yaml::to_string(&YamlValue::Mapping(yaml_map))?
        }
        ConfigFormat::Toml => {
            let mut toml_map = toml::map::Map::new();
            for (key, value) in properties {
                unflatten_toml(&mut toml_map, &key, value);
            }
            toml::to_string_pretty(&TomlValue::Table(toml_map))?
        }
        ConfigFormat::Json => {
            let mut json_obj = serde_json::Map::new();
            for (key, value) in properties {
                unflatten_json(&mut json_obj, &key, value);
            }
            serde_json::to_string_pretty(&JsonValue::Object(json_obj))?
        }
    };

    fs::write(&full_path, content).await
        .context(format!("Failed to write config file: {}", rel_path))?;
    
    Ok(())
}

fn flatten_yaml(prefix: &str, value: &YamlValue, props: &mut HashMap<String, String>) {
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

fn flatten_toml(prefix: &str, value: &TomlValue, props: &mut HashMap<String, String>) {
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

fn flatten_json(prefix: &str, value: &JsonValue, props: &mut HashMap<String, String>) {
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

fn unflatten_yaml(map: &mut serde_yaml::Mapping, key: &str, value: String) {
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

fn unflatten_toml(map: &mut toml::map::Map<String, TomlValue>, key: &str, value: String) {
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

fn unflatten_json(map: &mut serde_json::Map<String, JsonValue>, key: &str, value: String) {
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
