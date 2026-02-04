use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context};
use std::io::Read;
use std::collections::HashMap;
use serde::Deserialize;
use base64::{Engine as _, engine::general_purpose};
use crate::mods::types::{InstalledMod, ModCache, ModCacheEntry};

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
                    extract_metadata_sync(&path_clone)
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
