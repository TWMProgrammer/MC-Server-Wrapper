use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};
use tokio::fs;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use ::toml::Value as TomlValue;
use super::types::ConfigFormat;
use super::{properties, yaml, toml, json};

pub async fn read_config_file(instance_path: &Path, rel_path: &str, format: ConfigFormat) -> Result<HashMap<String, String>> {
    let full_path = instance_path.join(rel_path);
    if !full_path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&full_path).await
        .context(format!("Failed to read config file: {}", rel_path))?;

    match format {
        ConfigFormat::Properties => {
            Ok(properties::parse_properties(&content))
        }
        ConfigFormat::Yaml => {
            let yaml: YamlValue = serde_yaml::from_str(&content)?;
            let mut props = HashMap::new();
            yaml::flatten_yaml("", &yaml, &mut props);
            Ok(props)
        }
        ConfigFormat::Toml => {
            let toml: TomlValue = ::toml::from_str(&content)?;
            let mut props = HashMap::new();
            toml::flatten_toml("", &toml, &mut props);
            Ok(props)
        }
        ConfigFormat::Json => {
            let json: JsonValue = serde_json::from_str(&content)?;
            let mut props = HashMap::new();
            json::flatten_json("", &json, &mut props);
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
            Ok(properties::parse_properties_as_json(&content))
        }
        ConfigFormat::Yaml => {
            let yaml: YamlValue = serde_yaml::from_str(&content)?;
            let json_str = serde_json::to_string(&yaml)?;
            let json: JsonValue = serde_json::from_str(&json_str)?;
            Ok(json)
        }
        ConfigFormat::Toml => {
            let toml: TomlValue = ::toml::from_str(&content)?;
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
            properties::serialize_json_as_properties(&value)
        }
        ConfigFormat::Yaml => {
            let yaml: YamlValue = serde_json::from_value(value)?;
            serde_yaml::to_string(&yaml)?
        }
        ConfigFormat::Toml => {
            let toml: TomlValue = serde_json::from_value(value)?;
            ::toml::to_string_pretty(&toml)?
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
            properties::serialize_properties(&properties)
        }
        ConfigFormat::Yaml => {
            let mut yaml_map = serde_yaml::Mapping::new();
            for (key, value) in properties {
                yaml::unflatten_yaml(&mut yaml_map, &key, value);
            }
            serde_yaml::to_string(&YamlValue::Mapping(yaml_map))?
        }
        ConfigFormat::Toml => {
            let mut toml_map = ::toml::map::Map::new();
            for (key, value) in properties {
                toml::unflatten_toml(&mut toml_map, &key, value);
            }
            ::toml::to_string_pretty(&TomlValue::Table(toml_map))?
        }
        ConfigFormat::Json => {
            let mut json_obj = serde_json::Map::new();
            for (key, value) in properties {
                json::unflatten_json(&mut json_obj, &key, value);
            }
            serde_json::to_string_pretty(&JsonValue::Object(json_obj))?
        }
    };

    fs::write(&full_path, content).await
        .context(format!("Failed to write config file: {}", rel_path))?;
    
    Ok(())
}
