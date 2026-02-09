use std::path::Path;
use tokio::fs;
use super::types::{ConfigFile, ConfigFormat};

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
        ("velocity.toml", ConfigFormat::Toml),
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
