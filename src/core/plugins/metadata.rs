use std::path::Path;
use std::io::Read;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use super::types::{InstalledPlugin, PluginSource};

#[derive(Debug, Deserialize)]
pub struct PluginYml {
    pub name: String,
    pub version: Option<serde_yaml::Value>,
    pub author: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginCacheEntry {
    pub last_modified: u64,
    pub metadata: InstalledPlugin,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PluginCache {
    pub entries: HashMap<String, PluginCacheEntry>,
    pub sources: HashMap<String, PluginSource>,
}

/// Extracts metadata from a plugin JAR file.
pub fn extract_metadata_sync(path: &Path) -> Result<InstalledPlugin> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let is_disabled = filename.ends_with(".disabled");
    
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    let mut content = String::new();
    let mut found = false;
    
    // Check for common plugin metadata files
    for filename_in_zip in ["plugin.yml", "bungee.yml", "paper-plugin.yml"] {
        if let Ok(mut file) = archive.by_name(filename_in_zip) {
            file.read_to_string(&mut content)?;
            found = true;
            break;
        }
    }
    
    if !found {
        // Fallback to filename-based name
        let name = if is_disabled {
            filename.strip_suffix(".jar.disabled").unwrap_or(&filename).to_string()
        } else {
            filename.strip_suffix(".jar").unwrap_or(&filename).to_string()
        };
        return Ok(InstalledPlugin {
            name,
            filename,
            enabled: !is_disabled,
            version: None,
            author: None,
            description: None,
            source: None,
        });
    }
    
    // Parse YAML, but be lenient with errors
    let yaml: PluginYml = match serde_yaml::from_str(&content) {
        Ok(y) => y,
        Err(_) => {
            // If parsing fails, return basic info
            let name = if is_disabled {
                filename.strip_suffix(".jar.disabled").unwrap_or(&filename).to_string()
            } else {
                filename.strip_suffix(".jar").unwrap_or(&filename).to_string()
            };
            return Ok(InstalledPlugin {
                name,
                filename,
                enabled: !is_disabled,
                version: None,
                author: None,
                description: None,
                source: None,
            });
        }
    };
    
    let author = yaml.author.or_else(|| {
        yaml.authors.and_then(|a| if a.is_empty() { None } else { Some(a.join(", ")) })
    });

    let version = yaml.version.map(|v| match v {
        serde_yaml::Value::String(s) => s,
        serde_yaml::Value::Number(n) => n.to_string(),
        _ => "unknown".to_string(),
    });

    Ok(InstalledPlugin {
        name: yaml.name,
        filename,
        enabled: !is_disabled,
        version,
        author,
        description: yaml.description,
        source: None,
    })
}
