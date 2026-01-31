pub mod fabric;
pub mod forge;
pub mod neoforge;
pub mod paper;
pub mod purpur;

use serde::{Deserialize, Serialize};
use anyhow::Result;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLoader {
    pub name: String,
    pub versions: Vec<String>,
}

pub struct ModLoaderClient {
    pub(crate) client: reqwest::Client,
}

impl ModLoaderClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    pub(crate) async fn download_with_progress<F>(&self, url: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut file = tokio::fs::File::create(target_path).await?;
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
            _ => Err(anyhow::anyhow!("Unsupported mod loader: {}", loader_name)),
        }
    }

    pub async fn get_available_loaders(&self, mc_version: &str) -> Result<Vec<ModLoader>> {
        let mut loaders = Vec::new();

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

        Ok(loaders)
    }
}
