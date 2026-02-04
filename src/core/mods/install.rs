use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use anyhow::{Result, Context, anyhow};
use crate::mods::types::{ModProvider, ProjectVersion, ModCache, ModSource, ModUpdate};
use crate::mods::modrinth::ModrinthClient;
use crate::mods::curseforge::CurseForgeClient;
use crate::mods::metadata::list_installed_mods;
use crate::cache::CacheManager;

/// Uninstalls a mod by removing its file and optionally its configuration folder.
pub async fn uninstall_mod(instance_path: impl AsRef<Path>, filename: String, delete_config: bool) -> Result<()> {
    // Path traversal protection
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(anyhow::anyhow!("Invalid filename: {}", filename));
    }

    let mods_dir = instance_path.as_ref().join("mods");
    let mod_file = mods_dir.join(&filename);

    if mod_file.exists() {
        fs::remove_file(mod_file).await.context("Failed to delete mod file")?;
    }

    if delete_config {
        // Try to find the config directory. Mods usually put configs in the instance's 'config' directory.
        // We'll need a way to map mod IDs to config files in Phase 4.
        // For now, we'll just implement the placeholder logic.
        let instance_config_dir = instance_path.as_ref().join("config");
        if instance_config_dir.exists() {
            // TODO: In Phase 4, implement actual config discovery for mods.
        }
    }

    Ok(())
}

/// Uninstalls multiple mods at once.
pub async fn bulk_uninstall_mods(
    instance_path: impl AsRef<Path>,
    filenames: Vec<String>,
    delete_config: bool,
) -> Result<()> {
    for filename in filenames {
        uninstall_mod(&instance_path, filename, delete_config).await?;
    }
    Ok(())
}

pub async fn install_mod(
    instance_path: impl AsRef<Path>,
    project_id: &str,
    provider: ModProvider,
    version_id: Option<&str>,
    game_version: Option<&str>,
    loader: Option<&str>,
    curseforge_api_key: Option<String>,
    cache: Arc<CacheManager>,
) -> Result<String> {
    let mods_dir = instance_path.as_ref().join("mods");
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir).await?;
    }

    let (filename, final_version_id): (String, String) = match provider {
        ModProvider::Modrinth => {
            let client = ModrinthClient::new(cache);
            let versions: Vec<ProjectVersion> = client.get_versions(project_id, game_version, loader).await?;
            
            let version = if let Some(vid) = version_id {
                versions.iter().find(|v| v.id == vid)
                    .ok_or_else(|| anyhow!("Version not found: {}", vid))?
            } else {
                versions.first()
                    .ok_or_else(|| anyhow!("No versions found for project: {}", project_id))?
            };

            let fname = client.download_version(version, &mods_dir).await?;
            (fname, version.id.clone())
        }
        ModProvider::CurseForge => {
            let client = CurseForgeClient::new(curseforge_api_key, cache);
            let versions: Vec<ProjectVersion> = client.get_versions(project_id, game_version, loader).await?;
            
            let version = if let Some(vid) = version_id {
                versions.iter().find(|v| v.id == vid)
                    .ok_or_else(|| anyhow!("Version not found: {}", vid))?
            } else {
                versions.first()
                    .ok_or_else(|| anyhow!("No versions found for project: {}", project_id))?
            };

            let file = version.files.first().ok_or_else(|| anyhow!("No files found for version"))?;
            let fname = client.download_file(&file.url, &file.filename, &mods_dir).await?;
            (fname, version.id.clone())
        }
    };

    // Update cache with source info
    let cache_path = mods_dir.join(".mod_metadata_cache.json");
    let mut cache: ModCache = if cache_path.exists() {
        let content = fs::read_to_string(&cache_path).await.unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        ModCache::default()
    };

    cache.sources.insert(filename.clone(), ModSource {
        project_id: project_id.to_string(),
        provider,
        current_version_id: Some(final_version_id),
    });

    if let Ok(content) = serde_json::to_string(&cache) {
        let _ = fs::write(&cache_path, content).await;
    }

    Ok(filename)
}

/// Checks for updates for all installed mods that have source information.
pub async fn check_for_updates(
    instance_path: impl AsRef<Path>,
    game_version: Option<&str>,
    loader: Option<&str>,
    curseforge_api_key: Option<String>,
    cache: Arc<CacheManager>,
) -> Result<Vec<ModUpdate>> {
    let installed = list_installed_mods(&instance_path).await?;
    let mut updates = Vec::new();

    for mod_item in installed {
        if let Some(source) = mod_item.source {
            match source.provider {
                ModProvider::Modrinth => {
                    let client = ModrinthClient::new(Arc::clone(&cache));
                    if let Ok(versions) = client.get_versions(&source.project_id, game_version, loader).await {
                        if let Some(latest) = versions.first() {
                            if Some(latest.id.clone()) != source.current_version_id {
                                updates.push(ModUpdate {
                                    filename: mod_item.filename.clone(),
                                    current_version: mod_item.version.clone(),
                                    latest_version: latest.version_number.clone(),
                                    latest_version_id: latest.id.clone(),
                                    project_id: source.project_id.clone(),
                                    provider: source.provider,
                                });
                            }
                        }
                    }
                }
                ModProvider::CurseForge => {
                    let client = CurseForgeClient::new(curseforge_api_key.clone(), Arc::clone(&cache));
                    if let Ok(versions) = client.get_versions(&source.project_id, game_version, loader).await {
                        if let Some(latest) = versions.first() {
                            if Some(latest.id.clone()) != source.current_version_id {
                                updates.push(ModUpdate {
                                    filename: mod_item.filename.clone(),
                                    current_version: mod_item.version.clone(),
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

/// Updates a mod by downloading the new version and replacing the old one.
pub async fn update_mod(
    instance_path: impl AsRef<Path>,
    filename: String,
    project_id: String,
    provider: ModProvider,
    latest_version_id: String,
    game_version: Option<&str>,
    loader: Option<&str>,
    curseforge_api_key: Option<String>,
    cache: Arc<CacheManager>,
) -> Result<()> {
    let mods_dir = instance_path.as_ref().join("mods");
    let old_path = mods_dir.join(&filename);

    // 1. Create backup
    let backup_path = mods_dir.join(format!("{}.bak", filename));
    if old_path.exists() {
        fs::copy(&old_path, &backup_path).await.context("Failed to create backup")?;
    }

    // 2. Download new version
    match install_mod(
        &instance_path,
        &project_id,
        provider,
        Some(&latest_version_id),
        game_version,
        loader,
        curseforge_api_key,
        cache,
    ).await {
        Ok(new_filename) => {
            let mut final_filename = new_filename.clone();

            // 3. Preserve disabled state
            if filename.ends_with(".disabled") && !final_filename.ends_with(".disabled") {
                let current_new_path = mods_dir.join(&new_filename);
                let disabled_new_filename = format!("{}.disabled", new_filename);
                let disabled_new_path = mods_dir.join(&disabled_new_filename);
                
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
