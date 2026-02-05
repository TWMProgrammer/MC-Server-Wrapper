use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use anyhow::{Result, anyhow};
use crate::mods::types::{ModProvider, ProjectVersion, ModCache, ModSource};
use crate::mods::modrinth::ModrinthClient;
use crate::mods::curseforge::CurseForgeClient;
use crate::cache::CacheManager;

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
