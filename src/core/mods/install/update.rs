use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use anyhow::{Result, Context};
use crate::mods::types::{ModProvider, ModUpdate};
use crate::mods::modrinth::ModrinthClient;
use crate::mods::curseforge::CurseForgeClient;
use crate::mods::metadata::list_installed_mods;
use crate::cache::CacheManager;
use super::install::install_mod;

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
