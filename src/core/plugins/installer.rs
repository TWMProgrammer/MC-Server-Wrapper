use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context};
use super::types::*;
use super::modrinth::ModrinthClient;
use super::spiget::SpigetClient;
use super::metadata::PluginCache;

/// Installs a plugin from a provider.
pub async fn install_plugin(
    instance_path: impl AsRef<Path>,
    project_id: &str,
    provider: PluginProvider,
    version_id: Option<&str>,
    game_version: Option<&str>,
    loader: Option<&str>,
) -> Result<String> {
    let plugins_dir = instance_path.as_ref().join("plugins");
    
    let (filename, vid) = match provider {
        PluginProvider::Modrinth => {
            let client = ModrinthClient::new();
            let versions = client.get_versions(project_id, game_version, loader).await?;
            let version = if let Some(vid) = version_id {
                versions.iter().find(|v| v.id == vid)
                    .ok_or_else(|| anyhow::anyhow!("Version not found: {}", vid))?
            } else {
                // Filter versions by game version and loader if provided
                let filtered: Vec<&ProjectVersion> = versions.iter().filter(|v| {
                    let version_match = game_version.map_or(true, |gv| v.game_versions.contains(&gv.to_string()));
                    let loader_match = loader.map_or(true, |l| {
                        let l_lower = l.to_lowercase();
                        v.loaders.iter().any(|vl| vl.to_lowercase() == l_lower)
                    });
                    version_match && loader_match
                }).collect();

                filtered.first().copied().or_else(|| versions.first())
                    .ok_or_else(|| anyhow::anyhow!("No versions found for project"))?
            };
            let fname = client.download_version(version, &plugins_dir).await?;
            (fname, Some(version.id.clone()))
        }
        PluginProvider::Spiget => {
            let client = SpigetClient::new();
            let fname = client
                .download_resource(project_id, &plugins_dir, game_version, loader)
                .await?;
            (fname, None)
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
    match install_plugin(&instance_path, &project_id, provider, Some(&latest_version_id), None, None).await {
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
