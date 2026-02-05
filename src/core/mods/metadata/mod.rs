use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context};
use crate::mods::types::{InstalledMod, ModCache, ModCacheEntry};

pub mod parsers;

/// Lists all installed mods in the given instance path.
pub async fn list_installed_mods(instance_path: impl AsRef<Path>) -> Result<Vec<InstalledMod>> {
    let mods_dir = instance_path.as_ref().join("mods");
    
    if !mods_dir.exists() {
        return Ok(vec![]);
    }

    // Load cache
    let cache_path = mods_dir.join(".mod_metadata_cache.json");
    let mut cache: ModCache = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path).await.unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ModCache::default()
    };

    let mut mods = Vec::new();
    let mut entries = fs::read_dir(&mods_dir).await.context("Failed to read mods directory")?;
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

                let source_key = if is_disabled {
                    filename.strip_suffix(".disabled").unwrap_or(&filename).to_string()
                } else {
                    filename.clone()
                };

                // Check cache
                if let Some(entry) = cache.entries.get(&filename) {
                    if entry.last_modified == last_modified {
                        let mut m = entry.metadata.clone();
                        m.enabled = !is_disabled;
                        m.source = cache.sources.get(&filename)
                            .or_else(|| cache.sources.get(&source_key))
                            .cloned();
                        mods.push(m);
                        continue;
                    }
                }

                // Extract metadata in a blocking task
                let path_clone = path.clone();
                let mut mod_item = tokio::task::spawn_blocking(move || {
                    parsers::extract_metadata_sync(&path_clone)
                }).await??;
                
                mod_item.source = cache.sources.get(&filename)
                    .or_else(|| cache.sources.get(&source_key))
                    .cloned();

                cache.entries.insert(filename.clone(), ModCacheEntry {
                    last_modified,
                    metadata: mod_item.clone(),
                });
                cache_updated = true;
                mods.push(mod_item);
            }
        }
    }

    // Save cache if updated
    if cache_updated {
        if let Ok(content) = serde_json::to_string(&cache) {
            let _ = fs::write(&cache_path, content).await;
        }
    }

    Ok(mods)
}

/// Toggles a mod's enabled state by renaming the file.
pub async fn toggle_mod(instance_path: impl AsRef<Path>, filename: String, enable: bool) -> Result<()> {
    let mods_dir = instance_path.as_ref().join("mods");
    let current_path = mods_dir.join(&filename);
    
    if !current_path.exists() {
        return Err(anyhow::anyhow!("Mod file not found: {}", filename));
    }

    let new_filename = if enable {
        if !filename.ends_with(".jar.disabled") {
            return Ok(());
        }
        filename.strip_suffix(".disabled").unwrap().to_string()
    } else {
        if filename.ends_with(".jar.disabled") {
            return Ok(());
        }
        format!("{}.disabled", filename)
    };

    let new_path = mods_dir.join(new_filename);
    fs::rename(current_path, new_path).await.context("Failed to rename mod file")?;

    Ok(())
}

/// Toggles multiple mods at once.
pub async fn bulk_toggle_mods(
    instance_path: impl AsRef<Path>,
    filenames: Vec<String>,
    enable: bool,
) -> Result<()> {
    for filename in filenames {
        toggle_mod(&instance_path, filename, enable).await?;
    }
    Ok(())
}
