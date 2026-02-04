pub mod fabric;
pub mod forge;
pub mod neoforge;
pub mod paper;
pub mod purpur;
pub mod proxy;
pub mod bedrock;

use crate::utils::retry_async;
use crate::cache::CacheManager;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLoader {
    pub name: String,
    pub versions: Vec<String>,
}

pub struct ModLoaderClient {
    pub(crate) client: reqwest::Client,
    pub(crate) cache_dir: Option<PathBuf>,
    pub(crate) cache: Arc<CacheManager>,
}

impl ModLoaderClient {
    pub fn new(cache_dir: Option<PathBuf>, cache: Arc<CacheManager>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache_dir,
            cache,
        }
    }

    pub(crate) async fn download_with_progress<F>(&self, url: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let target_path_ref = target_path.as_ref();
        retry_async(
            || async {
                let response = self.client.get(url).send().await?;
                if !response.status().is_success() {
                    return Err(anyhow!("Failed to download: {}", response.status()));
                }

                let total_size = response.content_length().unwrap_or(0);
                let mut file = tokio::fs::File::create(target_path_ref).await?;
                let mut downloaded: u64 = 0;
                let mut stream = response.bytes_stream();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    file.write_all(&chunk).await?;
                    downloaded += chunk.len() as u64;
                    on_progress(downloaded, total_size);
                }

                file.flush().await?;
                Ok(())
            },
            3,
            Duration::from_secs(2),
            &format!("Download from {}", url)
        ).await
    }

    pub async fn download_loader<F>(&self, loader_name: &str, mc_version: &str, loader_version: Option<&str>, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        match loader_name.to_lowercase().as_str() {
            "paper" => {
                let build = loader_version.ok_or_else(|| anyhow::anyhow!("Paper requires a build number"))?;
                self.download_paper(mc_version, build, target_path, on_progress).await
            },
            "fabric" => {
                let version = loader_version.ok_or_else(|| anyhow::anyhow!("Fabric requires a loader version"))?;
                self.download_fabric(mc_version, version, target_path, on_progress).await
            },
            "forge" => {
                let version = loader_version.ok_or_else(|| anyhow::anyhow!("Forge requires a version"))?;
                self.download_forge(mc_version, version, target_path, on_progress).await
            },
            "purpur" => {
                let build = loader_version.ok_or_else(|| anyhow::anyhow!("Purpur requires a build number"))?;
                self.download_purpur(mc_version, build, target_path, on_progress).await
            },
            "neoforge" => {
                let version = loader_version.ok_or_else(|| anyhow::anyhow!("NeoForge requires a version"))?;
                self.download_neoforge(version, target_path, on_progress).await
            },
            "velocity" => {
                let version = mc_version;
                let build = match loader_version {
                    Some(v) => v.to_string(),
                    None => {
                        let builds = self.get_velocity_builds(version).await?;
                        builds.first()
                            .ok_or_else(|| anyhow::anyhow!("No builds found for Velocity version {}", version))?
                            .clone()
                    }
                };
                self.download_velocity(version, &build, target_path, on_progress).await
            },
            "bungeecord" => {
                let version = loader_version.unwrap_or("latest");
                self.download_bungeecord(version, target_path, on_progress).await
            },
            "bedrock" => {
                let version = mc_version;
                // For Bedrock, target_path is the directory where it should be extracted
                let target_dir = target_path.as_ref().parent().ok_or_else(|| anyhow::anyhow!("Invalid target path for Bedrock"))?;
                self.download_bedrock(version, target_dir, on_progress).await
            },
            _ => Err(anyhow::anyhow!("Unsupported mod loader: {}", loader_name)),
        }
    }

    pub async fn get_available_loaders(&self, mc_version: &str, server_type: Option<&str>) -> Result<Vec<ModLoader>> {
        let mut loaders = Vec::new();

        // If we know the server type, we can skip the bedrock check if it's not bedrock
        let is_bedrock = if let Some(t) = server_type {
            t.to_lowercase() == "bedrock"
        } else {
            let bedrock_manifest = self.get_bedrock_versions().await.unwrap_or_else(|_| crate::downloader::VersionManifest {
                latest: crate::downloader::LatestVersions { release: "".to_string(), snapshot: "".to_string() },
                versions: vec![],
            });
            bedrock_manifest.versions.iter().any(|v| v.id == mc_version)
        };

        if is_bedrock {
            // This is a Bedrock version, don't return Java loaders
            return Ok(vec![]);
        }

        // Fabric
        if let Ok(versions) = self.get_fabric_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Fabric".to_string(),
                    versions,
                });
            }
        }

        // Forge
        if let Ok(versions) = self.get_forge_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Forge".to_string(),
                    versions,
                });
            }
        }

        // NeoForge
        if let Ok(versions) = self.get_neoforge_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "NeoForge".to_string(),
                    versions,
                });
            }
        }

        // Paper
        if let Ok(versions) = self.get_paper_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Paper".to_string(),
                    versions,
                });
            }
        }

        // Purpur
        if let Ok(versions) = self.get_purpur_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Purpur".to_string(),
                    versions,
                });
            }
        }

        // Proxies (Velocity/BungeeCord)
        if let Ok(versions) = self.get_velocity_versions().await {
            loaders.push(ModLoader {
                name: "Velocity".to_string(),
                versions,
            });
        }

        if let Ok(versions) = self.get_bungeecord_versions().await {
            loaders.push(ModLoader {
                name: "BungeeCord".to_string(),
                versions,
            });
        }

        Ok(loaders)
    }
}
