pub mod types;
pub mod modrinth;
pub mod spiget;
pub mod curseforge;

pub use types::*;
pub use modrinth::*;
pub use spiget::*;
pub use curseforge::*;

use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context};
use std::io::Read;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct PluginYml {
    name: String,
    version: Option<serde_yaml::Value>,
    author: Option<String>,
    authors: Option<Vec<String>>,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PluginCacheEntry {
    last_modified: u64,
    metadata: InstalledPlugin,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct PluginCache {
    entries: HashMap<String, PluginCacheEntry>,
    sources: HashMap<String, PluginSource>,
}

/// Extracts metadata from a plugin JAR file.
fn extract_metadata_sync(path: &Path) -> Result<InstalledPlugin> {
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

/// Lists all installed plugins in the given instance path.
pub async fn list_installed_plugins(instance_path: impl AsRef<Path>) -> Result<Vec<InstalledPlugin>> {
    let plugins_dir = instance_path.as_ref().join("plugins");
    
    if !plugins_dir.exists() {
        return Ok(vec![]);
    }

    // Load cache
    let cache_path = plugins_dir.join(".plugin_metadata_cache.json");
    let mut cache: PluginCache = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path).await.unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        PluginCache::default()
    };

    let mut plugins = Vec::new();
    let mut entries = fs::read_dir(&plugins_dir).await.context("Failed to read plugins directory")?;
    let mut cache_updated = false;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            let filename = entry.file_name().to_string_lossy().to_string();
            let is_jar = filename.to_lowercase().ends_with(".jar");
            let is_disabled = filename.to_lowercase().ends_with(".jar.disabled");

            if is_jar || is_disabled {
                let metadata = fs::metadata(&path).await?;
                let last_modified = metadata.modified()?
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs();

                // Check cache
                if let Some(entry) = cache.entries.get(&filename) {
                    if entry.last_modified == last_modified {
                        let mut p = entry.metadata.clone();
                        p.enabled = !is_disabled; // Update enabled state just in case it was renamed
                        p.source = cache.sources.get(&filename).cloned();
                        plugins.push(p);
                        continue;
                    }
                }

                // Extract metadata in a blocking task
                let path_clone = path.clone();
                let mut plugin = tokio::task::spawn_blocking(move || {
                    extract_metadata_sync(&path_clone)
                }).await??;
                
                plugin.source = cache.sources.get(&filename).cloned();

                cache.entries.insert(filename.clone(), PluginCacheEntry {
                    last_modified,
                    metadata: plugin.clone(),
                });
                cache_updated = true;
                plugins.push(plugin);
            }
        }
    }

    // Save cache if updated
    if cache_updated {
        if let Ok(content) = serde_json::to_string(&cache) {
            let _ = fs::write(&cache_path, content).await;
        }
    }

    Ok(plugins)
}

/// Toggles a plugin's enabled state by renaming the file.
pub async fn toggle_plugin(instance_path: impl AsRef<Path>, filename: String, enable: bool) -> Result<()> {
    let plugins_dir = instance_path.as_ref().join("plugins");
    let current_path = plugins_dir.join(&filename);
    
    if !current_path.exists() {
        return Err(anyhow::anyhow!("Plugin file not found: {}", filename));
    }

    let new_filename = if enable {
        if !filename.ends_with(".jar.disabled") {
            return Ok(()); // Already enabled or not a disabled plugin
        }
        filename.strip_suffix(".disabled").unwrap().to_string()
    } else {
        if filename.ends_with(".jar.disabled") {
            return Ok(()); // Already disabled
        }
        format!("{}.disabled", filename)
    };

    let new_path = plugins_dir.join(new_filename);
    fs::rename(current_path, new_path).await.context("Failed to rename plugin file")?;

    Ok(())
}

/// Searches for plugins across multiple providers.
pub async fn search_plugins(options: &SearchOptions, provider: Option<PluginProvider>) -> Result<Vec<Project>> {
    let mut results = Vec::new();

    match provider {
        Some(PluginProvider::Modrinth) => {
            let client = ModrinthClient::new();
            results.extend(client.search(options).await?);
        }
        Some(PluginProvider::Spiget) => {
            let client = SpigetClient::new();
            results.extend(client.search(options).await?);
        }
        Some(PluginProvider::CurseForge) => {
            // CurseForge requires an API key, which we don't handle globally yet
            // Return empty or error if explicitly requested
            return Err(anyhow::anyhow!("CurseForge search is not yet implemented"));
        }
        None => {
            // Search all providers
            let modrinth = ModrinthClient::new();
            let spiget = SpigetClient::new();

            let (m_res, s_res) = tokio::join!(
                modrinth.search(options),
                spiget.search(options)
            );

            if let Ok(res) = m_res { results.extend(res); }
            if let Ok(res) = s_res { results.extend(res); }
        }
    }

    Ok(results)
}

/// Gets dependencies for a plugin.
pub async fn get_plugin_dependencies(project_id: &str, provider: PluginProvider) -> Result<Vec<Project>> {
    match provider {
        PluginProvider::Modrinth => {
            let client = ModrinthClient::new();
            client.get_dependencies(project_id).await
        }
        _ => Ok(vec![]), // Spiget and CurseForge dependencies not implemented yet
    }
}

/// Installs a plugin from a provider.
pub async fn install_plugin(
    instance_path: impl AsRef<Path>,
    project_id: &str,
    provider: PluginProvider,
    version_id: Option<&str>,
) -> Result<String> {
    let plugins_dir = instance_path.as_ref().join("plugins");
    
    let (filename, vid) = match provider {
        PluginProvider::Modrinth => {
            let client = ModrinthClient::new();
            let versions = client.get_versions(project_id).await?;
            let version = if let Some(vid) = version_id {
                versions.iter().find(|v| v.id == vid)
                    .ok_or_else(|| anyhow::anyhow!("Version not found: {}", vid))?
            } else {
                versions.first().ok_or_else(|| anyhow::anyhow!("No versions found for project"))?
            };
            let fname = client.download_version(version, &plugins_dir).await?;
            (fname, Some(version.id.clone()))
        }
        PluginProvider::Spiget => {
            let client = SpigetClient::new();
            let fname = client.download_resource(project_id, &plugins_dir).await?;
            (fname, None)
        }
        PluginProvider::CurseForge => {
            return Err(anyhow::anyhow!("CurseForge installation is not yet implemented"));
        }
    };

    // Update source cache
    let cache_path = plugins_dir.join(".plugin_metadata_cache.json");
    let mut cache: PluginCache = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path).await.unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        PluginCache::default()
    };

    cache.sources.insert(filename.clone(), PluginSource {
        project_id: project_id.to_string(),
        provider,
        current_version_id: vid,
    });

    if let Ok(content) = serde_json::to_string(&cache) {
        let _ = fs::write(&cache_path, content).await;
    }

    Ok(filename)
}

/// Uninstalls a plugin by removing its file and optionally its configuration folder.
pub async fn uninstall_plugin(instance_path: impl AsRef<Path>, filename: String, delete_config: bool) -> Result<()> {
    let plugins_dir = instance_path.as_ref().join("plugins");
    let plugin_file = plugins_dir.join(&filename);

    if plugin_file.exists() {
        fs::remove_file(plugin_file).await.context("Failed to delete plugin file")?;
    }

    if delete_config {
        // Try to find the config directory. Usually it matches the plugin name.
        let plugin_name = if filename.ends_with(".jar.disabled") {
            filename.strip_suffix(".jar.disabled").unwrap()
        } else {
            filename.strip_suffix(".jar").unwrap_or(&filename)
        };

        let config_dir = plugins_dir.join(plugin_name);
        if config_dir.is_dir() {
            fs::remove_dir_all(config_dir).await.context("Failed to delete plugin config directory")?;
        }
    }

    Ok(())
}

/// Toggles multiple plugins at once.
pub async fn bulk_toggle_plugins(
    instance_path: impl AsRef<Path>,
    filenames: Vec<String>,
    enable: bool,
) -> Result<()> {
    for filename in filenames {
        let _ = toggle_plugin(&instance_path, filename, enable).await;
    }
    Ok(())
}

/// Uninstalls multiple plugins at once.
pub async fn bulk_uninstall_plugins(
    instance_path: impl AsRef<Path>,
    filenames: Vec<String>,
    delete_config: bool,
) -> Result<()> {
    for filename in filenames {
        let _ = uninstall_plugin(&instance_path, filename, delete_config).await;
    }
    Ok(())
}

/// Checks for updates for all installed plugins that have source information.
pub async fn check_for_updates(instance_path: impl AsRef<Path>) -> Result<Vec<PluginUpdate>> {
    let installed = list_installed_plugins(&instance_path).await?;
    let mut updates = Vec::new();

    for plugin in installed {
        if let Some(source) = plugin.source {
            match source.provider {
                PluginProvider::Modrinth => {
                    let client = ModrinthClient::new();
                    if let Ok(versions) = client.get_versions(&source.project_id).await {
                        if let Some(latest) = versions.first() {
                            if Some(latest.id.clone()) != source.current_version_id {
                                updates.push(PluginUpdate {
                                    filename: plugin.filename.clone(),
                                    current_version: plugin.version.clone(),
                                    latest_version: latest.version_number.clone(),
                                    latest_version_id: latest.id.clone(),
                                    project_id: source.project_id.clone(),
                                    provider: source.provider,
                                });
                            }
                        }
                    }
                }
                PluginProvider::Spiget => {
                    let client = SpigetClient::new();
                    if let Ok((latest_id, latest_name)) = client.get_latest_version(&source.project_id).await {
                        if Some(latest_id.clone()) != source.current_version_id {
                            updates.push(PluginUpdate {
                                filename: plugin.filename.clone(),
                                current_version: plugin.version.clone(),
                                latest_version: latest_name,
                                latest_version_id: latest_id,
                                project_id: source.project_id.clone(),
                                provider: source.provider,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(updates)
}

/// Updates a plugin by downloading the new version and replacing the old one.
pub async fn update_plugin(
    instance_path: impl AsRef<Path>,
    filename: String,
    project_id: String,
    provider: PluginProvider,
    latest_version_id: String,
) -> Result<()> {
    let plugins_dir = instance_path.as_ref().join("plugins");
    let old_path = plugins_dir.join(&filename);

    // 1. Create backup
    let backup_path = plugins_dir.join(format!("{}.bak", filename));
    if old_path.exists() {
        fs::copy(&old_path, &backup_path).await.context("Failed to create backup")?;
    }

    // 2. Download new version
    match install_plugin(&instance_path, &project_id, provider, Some(&latest_version_id)).await {
        Ok(new_filename) => {
            let mut final_filename = new_filename.clone();

            // 3. Preserve disabled state
            if filename.ends_with(".disabled") && !final_filename.ends_with(".disabled") {
                let current_new_path = plugins_dir.join(&new_filename);
                let disabled_new_filename = format!("{}.disabled", new_filename);
                let disabled_new_path = plugins_dir.join(&disabled_new_filename);
                
                if let Ok(_) = fs::rename(current_new_path, disabled_new_path).await {
                    final_filename = disabled_new_filename;
                }
            }

            // 4. If the filename changed (and we didn't already handle it by renaming above), delete the old one
            if final_filename != filename && old_path.exists() {
                let _ = fs::remove_file(old_path).await;
            }

            // 5. Delete backup on success
            let _ = fs::remove_file(backup_path).await;
            Ok(())
        }
        Err(e) => {
            // Restore from backup if download failed
            if backup_path.exists() {
                let _ = fs::rename(backup_path, old_path).await;
            }
            Err(e)
        }
    }
}
