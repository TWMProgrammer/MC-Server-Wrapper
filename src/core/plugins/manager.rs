use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context};
use super::types::*;
use super::modrinth::ModrinthClient;
use super::spiget::SpigetClient;
use super::hangar::HangarClient;
use super::metadata::{PluginCache, PluginCacheEntry, extract_metadata_sync};

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
        Some(PluginProvider::Hangar) => {
            let client = HangarClient::new();
            results.extend(client.search(options).await?);
        }
        None => {
            // Search all providers
            let modrinth = ModrinthClient::new();
            let spiget = SpigetClient::new();
            let hangar = HangarClient::new();

            let (m_res, s_res, h_res) = tokio::join!(
                modrinth.search(options),
                spiget.search(options),
                hangar.search(options)
            );

            if let Ok(res) = m_res { results.extend(res); }
            if let Ok(res) = s_res { results.extend(res); }
            if let Ok(res) = h_res { results.extend(res); }
        }
    }

    Ok(results)
}

/// Gets dependencies for a plugin.
pub async fn get_plugin_dependencies(project_id: &str, provider: PluginProvider) -> Result<Vec<ResolvedDependency>> {
    match provider {
        PluginProvider::Modrinth => {
            let client = ModrinthClient::new();
            client.get_dependencies(project_id).await
        }
        PluginProvider::Spiget => {
            let client = SpigetClient::new();
            client.get_dependencies(project_id).await
        }
        PluginProvider::Hangar => {
            let client = HangarClient::new();
            client.get_dependencies(project_id).await
        }
    }
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

/// Checks for updates for all installed plugins that have source information.
pub async fn check_for_updates(
    instance_path: impl AsRef<Path>,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> Result<Vec<PluginUpdate>> {
    let installed = list_installed_plugins(&instance_path).await?;
    let mut updates = Vec::new();

    for plugin in installed {
        if let Some(source) = plugin.source {
            match source.provider {
                PluginProvider::Modrinth => {
                    let client = ModrinthClient::new();
                    if let Ok(versions) = client.get_versions(&source.project_id, game_version, loader).await {
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
                PluginProvider::Hangar => {
                    let client = HangarClient::new();
                    if let Ok(versions) = client.get_versions(&source.project_id, game_version, loader).await {
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
            }
        }
    }

    Ok(updates)
}
