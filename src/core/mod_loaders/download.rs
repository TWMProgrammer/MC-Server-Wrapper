use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use crate::utils::retry_async;
use super::client::ModLoaderClient;

impl ModLoaderClient {
    pub(crate) async fn download_with_progress<F>(
        &self,
        url: &str,
        target_path: impl AsRef<std::path::Path>,
        on_progress: F,
    ) -> Result<()>
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
            &format!("Download from {}", url),
        )
        .await
    }

    pub async fn download_loader<F>(
        &self,
        loader_name: &str,
        mc_version: &str,
        loader_version: Option<&str>,
        target_path: impl AsRef<std::path::Path>,
        on_progress: F,
    ) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        match loader_name.to_lowercase().as_str() {
            "paper" => {
                let build = match loader_version {
                    Some(v) => v.to_string(),
                    None => {
                        let builds = self.get_paper_versions(mc_version).await?;
                        builds
                            .first()
                            .ok_or_else(|| {
                                anyhow::anyhow!("No builds found for Paper version {}", mc_version)
                            })?
                            .clone()
                    }
                };
                self.download_paper(mc_version, &build, target_path, on_progress)
                    .await
            }
            "fabric" => {
                let version = loader_version
                    .ok_or_else(|| anyhow::anyhow!("Fabric requires a loader version"))?;
                self.download_fabric(mc_version, version, target_path, on_progress)
                    .await
            }
            "forge" => {
                let version =
                    loader_version.ok_or_else(|| anyhow::anyhow!("Forge requires a version"))?;
                self.download_forge(mc_version, version, target_path, on_progress)
                    .await
            }
            "purpur" => {
                let build = match loader_version {
                    Some(v) => v.to_string(),
                    None => {
                        let builds = self.get_purpur_versions(mc_version).await?;
                        builds
                            .first()
                            .ok_or_else(|| {
                                anyhow::anyhow!("No builds found for Purpur version {}", mc_version)
                            })?
                            .clone()
                    }
                };
                self.download_purpur(mc_version, &build, target_path, on_progress)
                    .await
            }
            "neoforge" => {
                let version =
                    loader_version.ok_or_else(|| anyhow::anyhow!("NeoForge requires a version"))?;
                self.download_neoforge(version, target_path, on_progress)
                    .await
            }
            "velocity" => {
                let version = mc_version;
                let build = match loader_version {
                    Some(v) => v.to_string(),
                    None => {
                        let builds = self.get_velocity_builds(version).await?;
                        builds
                            .first()
                            .ok_or_else(|| {
                                anyhow::anyhow!("No builds found for Velocity version {}", version)
                            })?
                            .clone()
                    }
                };
                self.download_velocity(version, &build, target_path, on_progress)
                    .await
            }
            "bungeecord" => {
                let version = loader_version.unwrap_or("latest");
                self.download_bungeecord(version, target_path, on_progress)
                    .await
            }
            "bedrock" => {
                let version = mc_version;
                // For Bedrock, target_path is the directory where it should be extracted
                let target_dir = target_path
                    .as_ref()
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("Invalid target path for Bedrock"))?;
                self.download_bedrock(version, target_dir, on_progress)
                    .await
            }
            _ => Err(anyhow::anyhow!("Unsupported mod loader: {}", loader_name)),
        }
    }
}
