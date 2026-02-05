use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context};
use crate::plugins::types::InstalledPlugin;
use crate::plugins::metadata::{PluginCache, PluginCacheEntry, extract_metadata_sync};

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
