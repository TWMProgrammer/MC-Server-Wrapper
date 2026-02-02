pub mod types;
pub mod curseforge;
pub mod modrinth;

pub use types::*;
pub use curseforge::*;
pub use modrinth::*;

use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context, anyhow};
use std::io::Read;
use std::collections::HashMap;
use serde::Deserialize;
use base64::{Engine as _, engine::general_purpose};

#[derive(Deserialize)]
struct FabricModJson {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    authors: Option<serde_json::Value>,
    icon: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct ModsToml {
    mods: Vec<ForgeModMetadata>,
}

#[derive(Deserialize)]
struct ForgeModMetadata {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    authors: Option<String>,
    #[serde(rename = "logoFile")]
    logo_file: Option<String>,
}

#[derive(Deserialize)]
struct QuiltModJson {
    quilt_loader: QuiltLoaderMetadata,
}

#[derive(Deserialize)]
struct QuiltLoaderMetadata {
    metadata: Option<QuiltMetadata>,
}

#[derive(Deserialize)]
struct QuiltMetadata {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    contributors: Option<HashMap<String, String>>,
    icon: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct LegacyForgeModInfo {
    #[serde(rename = "modList")]
    mod_list: Option<Vec<LegacyForgeMetadata>>,
}

#[derive(Deserialize)]
struct LegacyForgeMetadata {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author_list: Option<Vec<String>>,
}

/// Extracts basic metadata from a mod JAR file.
fn extract_metadata_sync(path: &Path) -> Result<InstalledMod> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let is_disabled = filename.ends_with(".disabled");
    
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut mod_item = InstalledMod {
        name: filename.clone(),
        filename: filename.clone(),
        enabled: !is_disabled,
        version: None,
        author: None,
        description: None,
        loader: None,
        source: None,
        icon_data: None,
    };

    // Try Fabric
    let mut icon_path = None;
    if let Ok(mut fabric_file) = archive.by_name("fabric.mod.json") {
        let mut content = String::new();
        fabric_file.read_to_string(&mut content)?;
        if let Ok(json) = serde_json::from_str::<FabricModJson>(&content) {
            mod_item.name = json.name.unwrap_or(mod_item.name);
            mod_item.version = json.version;
            mod_item.description = json.description;
            mod_item.loader = Some("Fabric".to_string());
            
            if let Some(authors) = json.authors {
                if authors.is_array() {
                    let author_names: Vec<String> = authors.as_array().unwrap().iter()
                        .filter_map(|a| {
                            if a.is_string() {
                                Some(a.as_str().unwrap().to_string())
                            } else if a.is_object() {
                                a.get("name").and_then(|n| n.as_str()).map(|n| n.to_string())
                            } else {
                                None
                            }
                        }).collect();
                    mod_item.author = Some(author_names.join(", "));
                } else if authors.is_string() {
                    mod_item.author = Some(authors.as_str().unwrap().to_string());
                }
            }

            if let Some(icon) = json.icon {
                icon_path = if icon.is_string() {
                    icon.as_str().map(|s| s.to_string())
                } else if icon.is_object() {
                    icon.get("128").or_else(|| icon.get("64")).or_else(|| icon.get("32"))
                        .and_then(|v| v.as_str()).map(|s| s.to_string())
                } else {
                    None
                };
            }
        }
    } else if let Ok(mut neoforge_file) = archive.by_name("META-INF/neoforge.mods.toml") {
        let mut content = String::new();
        neoforge_file.read_to_string(&mut content)?;
        if let Ok(toml_data) = toml::from_str::<ModsToml>(&content) {
            if let Some(first_mod) = toml_data.mods.first() {
                mod_item.name = first_mod.display_name.clone().unwrap_or(mod_item.name);
                mod_item.version = first_mod.version.clone();
                mod_item.description = first_mod.description.clone();
                mod_item.author = first_mod.authors.clone();
                mod_item.loader = Some("NeoForge".to_string());
                icon_path = first_mod.logo_file.clone();
            }
        }
    }

    if let Some(ip) = icon_path {
        if let Ok(mut icon_file) = archive.by_name(&ip) {
            let mut buffer = Vec::new();
            icon_file.read_to_end(&mut buffer)?;
            mod_item.icon_data = Some(general_purpose::STANDARD.encode(buffer));
        }
    }

    if mod_item.loader.is_none() {
        // Try Quilt
        let mut quilt_icon_path = None;
        if let Ok(mut quilt_file) = archive.by_name("quilt.mod.json") {
            let mut content = String::new();
            quilt_file.read_to_string(&mut content)?;
            if let Ok(json) = serde_json::from_str::<QuiltModJson>(&content) {
                if let Some(metadata) = json.quilt_loader.metadata {
                    mod_item.name = metadata.name.unwrap_or(mod_item.name);
                    mod_item.version = metadata.version;
                    mod_item.description = metadata.description;
                    mod_item.loader = Some("Quilt".to_string());

                    if let Some(contributors) = metadata.contributors {
                        let author_names: Vec<String> = contributors.keys().cloned().collect();
                        mod_item.author = Some(author_names.join(", "));
                    }

                    if let Some(icon) = metadata.icon {
                        quilt_icon_path = if icon.is_string() {
                            icon.as_str().map(|s| s.to_string())
                        } else if icon.is_object() {
                            icon.get("128").or_else(|| icon.get("64")).or_else(|| icon.get("32"))
                                .and_then(|v| v.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        };
                    }
                }
            }
        }

        if let Some(ip) = quilt_icon_path {
            if let Ok(mut icon_file) = archive.by_name(&ip) {
                let mut buffer = Vec::new();
                icon_file.read_to_end(&mut buffer)?;
                mod_item.icon_data = Some(general_purpose::STANDARD.encode(buffer));
            }
        }
    }

    if mod_item.loader.is_none() {
        // Try Forge/NeoForge (mods.toml)
        let mut forge_icon_path = None;
        if let Ok(mut forge_file) = archive.by_name("META-INF/mods.toml") {
            let mut content = String::new();
            forge_file.read_to_string(&mut content)?;
            if let Ok(toml_data) = toml::from_str::<ModsToml>(&content) {
                if let Some(first_mod) = toml_data.mods.first() {
                    mod_item.name = first_mod.display_name.clone().unwrap_or(mod_item.name);
                    mod_item.version = first_mod.version.clone();
                    mod_item.description = first_mod.description.clone();
                    mod_item.author = first_mod.authors.clone();
                    mod_item.loader = Some("Forge".to_string());
                    forge_icon_path = first_mod.logo_file.clone();
                }
            }
        }

        if let Some(logo) = forge_icon_path {
            if let Ok(mut icon_file) = archive.by_name(&logo) {
                let mut buffer = Vec::new();
                icon_file.read_to_end(&mut buffer)?;
                mod_item.icon_data = Some(general_purpose::STANDARD.encode(buffer));
            }
        }
    }

    if mod_item.loader.is_none() {
        // Try Legacy Forge (mcmod.info)
        if let Ok(mut legacy_file) = archive.by_name("mcmod.info") {
            let mut content = String::new();
            legacy_file.read_to_string(&mut content)?;
            // mcmod.info can be a list or a wrapped list
            if let Ok(mods) = serde_json::from_str::<Vec<LegacyForgeMetadata>>(&content) {
                if let Some(first_mod) = mods.first() {
                    mod_item.name = first_mod.name.clone().unwrap_or(mod_item.name);
                    mod_item.version = first_mod.version.clone();
                    mod_item.description = first_mod.description.clone();
                    mod_item.author = first_mod.author_list.as_ref().map(|l| l.join(", "));
                    mod_item.loader = Some("Forge".to_string());
                }
            } else if let Ok(wrapped) = serde_json::from_str::<LegacyForgeModInfo>(&content) {
                if let Some(mods) = wrapped.mod_list {
                    if let Some(first_mod) = mods.first() {
                        mod_item.name = first_mod.name.clone().unwrap_or(mod_item.name);
                        mod_item.version = first_mod.version.clone();
                        mod_item.description = first_mod.description.clone();
                        mod_item.author = first_mod.author_list.as_ref().map(|l| l.join(", "));
                        mod_item.loader = Some("Forge".to_string());
                    }
                }
            }
        }
    }

    // Fallback name cleaning if still using filename
    if mod_item.name == filename {
        let name = if is_disabled {
            filename.strip_suffix(".jar.disabled").unwrap_or(&filename).to_string()
        } else {
            filename.strip_suffix(".jar").unwrap_or(&filename).to_string()
        };
        mod_item.name = name;
    }

    Ok(mod_item)
}

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

                // Check cache
                if let Some(entry) = cache.entries.get(&filename) {
                    if entry.last_modified == last_modified {
                        let mut m = entry.metadata.clone();
                        m.enabled = !is_disabled;
                        m.source = cache.sources.get(&filename).cloned();
                        mods.push(m);
                        continue;
                    }
                }

                // Extract metadata in a blocking task
                let path_clone = path.clone();
                let mut mod_item = tokio::task::spawn_blocking(move || {
                    extract_metadata_sync(&path_clone)
                }).await??;
                
                mod_item.source = cache.sources.get(&filename).cloned();

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

/// Uninstalls a mod by removing its file and optionally its configuration folder.
pub async fn uninstall_mod(instance_path: impl AsRef<Path>, filename: String, delete_config: bool) -> Result<()> {
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

/// Searches for mods across multiple providers.
pub async fn search_mods(options: &SearchOptions, provider: Option<ModProvider>, curseforge_api_key: Option<String>) -> Result<Vec<Project>> {
    let mut results = Vec::new();

    match provider {
        Some(ModProvider::Modrinth) => {
            let client = ModrinthClient::new();
            results.extend(client.search(options).await?);
        }
        Some(ModProvider::CurseForge) => {
            let client = CurseForgeClient::new(curseforge_api_key);
            results.extend(client.search(options).await?);
        }
        None => {
            let modrinth = ModrinthClient::new();
            let curseforge = CurseForgeClient::new(curseforge_api_key);

            let (m_res, c_res) = tokio::join!(
                modrinth.search(options),
                curseforge.search(options)
            );

            if let Ok(res) = m_res { results.extend(res); }
            if let Ok(res) = c_res { results.extend(res); }
        }
    }

    Ok(results)
}

/// Gets dependencies for a mod.
pub async fn get_mod_dependencies(project_id: &str, provider: ModProvider, curseforge_api_key: Option<String>) -> Result<Vec<Project>> {
    match provider {
        ModProvider::Modrinth => {
            let client = ModrinthClient::new();
            client.get_dependencies(project_id).await
        }
        ModProvider::CurseForge => {
            let client = CurseForgeClient::new(curseforge_api_key);
            client.get_dependencies(project_id).await
        }
    }
}

/// Lists all files in a mod's config directory/file.
pub async fn list_mod_config_files(instance_path: impl AsRef<Path>, rel_path: &str) -> Result<Vec<String>> {
    let full_path = instance_path.as_ref().join(rel_path);
    if !full_path.exists() {
        return Ok(vec![]);
    }

    if full_path.is_file() {
        if let Ok(rel) = full_path.strip_prefix(instance_path.as_ref().join("config")) {
            return Ok(vec![rel.to_string_lossy().to_string().replace('\\', "/")]);
        }
        return Ok(vec![rel_path.to_string()]);
    }

    let mut files = Vec::new();
    let mut stack = vec![full_path.clone()];

    while let Some(current_dir) = stack.pop() {
        let mut entries = fs::read_dir(&current_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                // Get path relative to the config directory root
                if let Ok(rel) = path.strip_prefix(instance_path.as_ref().join("config")) {
                    files.push(rel.to_string_lossy().to_string().replace('\\', "/"));
                }
            }
        }
    }

    Ok(files)
}
pub async fn get_mod_configs(instance_path: impl AsRef<Path>, mod_name: &str) -> Result<Vec<ModConfig>> {
    let config_dir = instance_path.as_ref().join("config");
    if !config_dir.exists() {
        return Ok(vec![]);
    }

    let mut configs = Vec::new();
    let mut entries = fs::read_dir(&config_dir).await?;

    // Try to find files/dirs that match the mod name or a common ID pattern
    // e.g., for "Fabric API", look for "fabric-api.json" or "fabric"
    let mod_id = mod_name.to_lowercase().replace(' ', "-");
    let mod_id_short = mod_id.split('-').next().unwrap_or(&mod_id);

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().to_string_lossy().to_lowercase();
        let is_match = file_name.contains(&mod_id) || 
                      file_name.contains(mod_id_short) ||
                      mod_id.contains(&file_name);

        if is_match {
            let file_type = entry.file_type().await?;
            configs.push(ModConfig {
                name: entry.file_name().to_string_lossy().to_string(),
                path: format!("config/{}", entry.file_name().to_string_lossy()),
                is_dir: file_type.is_dir(),
            });
        }
    }

    Ok(configs)
}
pub async fn install_mod(
    instance_path: impl AsRef<Path>,
    project_id: &str,
    provider: ModProvider,
    version_id: Option<&str>,
    game_version: Option<&str>,
    loader: Option<&str>,
    curseforge_api_key: Option<String>,
) -> Result<String> {
    let mods_dir = instance_path.as_ref().join("mods");
    if !mods_dir.exists() {
        fs::create_dir_all(&mods_dir).await?;
    }

    let filename = match provider {
        ModProvider::Modrinth => {
            let client = ModrinthClient::new();
            let versions: Vec<ProjectVersion> = client.get_versions(project_id).await?;
            
            let version = if let Some(vid) = version_id {
                versions.iter().find(|v| v.id == vid)
                    .ok_or_else(|| anyhow!("Version not found: {}", vid))?
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
                    .ok_or_else(|| anyhow!("No versions found for project: {}", project_id))?
            };

            client.download_version(version, &mods_dir).await?
        }
        ModProvider::CurseForge => {
            let client = CurseForgeClient::new(curseforge_api_key);
            let versions: Vec<ProjectVersion> = client.get_versions(project_id).await?;
            
            let version = if let Some(vid) = version_id {
                versions.iter().find(|v| v.id == vid)
                    .ok_or_else(|| anyhow!("Version not found: {}", vid))?
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
                    .ok_or_else(|| anyhow!("No versions found for project: {}", project_id))?
            };

            let file = version.files.first().ok_or_else(|| anyhow!("No files found for version"))?;
            client.download_file(&file.url, &file.filename, &mods_dir).await?
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
        current_version_id: version_id.map(|s| s.to_string()),
    });

    if let Ok(content) = serde_json::to_string(&cache) {
        let _ = fs::write(&cache_path, content).await;
    }

    Ok(filename)
}
