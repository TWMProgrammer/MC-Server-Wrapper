pub mod types;

pub use types::*;

use crate::cache::CacheManager;
use crate::artifacts::{ArtifactStore, HashAlgorithm};
use crate::utils::retry_async;
use anyhow::{Result, anyhow, Context};
use futures_util::StreamExt;
use sha1::{Digest, Sha1};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug};

const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

pub struct VersionDownloader {
    client: reqwest::Client,
    cache_dir: Option<PathBuf>,
    cache: Option<Arc<CacheManager>>,
    artifact_store: Option<Arc<ArtifactStore>>,
}

impl VersionDownloader {
    pub fn new(
        cache_dir: Option<PathBuf>,
        cache: Option<Arc<CacheManager>>,
        artifact_store: Option<Arc<ArtifactStore>>,
    ) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache_dir,
            cache,
            artifact_store,
        }
    }

    pub async fn fetch_manifest(&self) -> Result<VersionManifest> {
        if let Some(ref cache) = self.cache {
            let client = self.client.clone();
            return cache
                .fetch_with_cache(
                    "mojang_version_manifest".to_string(),
                    Duration::from_secs(3600),
                    move || {
                        let client = client.clone();
                        async move {
                            info!("Fetching version manifest from {}", VERSION_MANIFEST_URL);
                            retry_async(
                                || async {
                                    client
                                        .get(VERSION_MANIFEST_URL)
                                        .send()
                                        .await?
                                        .json::<VersionManifest>()
                                        .await
                                        .map_err(|e| anyhow!(e))
                                },
                                3,
                                Duration::from_secs(2),
                                "Fetch version manifest",
                            )
                            .await
                        }
                    },
                )
                .await;
        }

        // Fallback to legacy manual caching if CacheManager is not available
        if let Some(cache_dir) = &self.cache_dir {
            let cache_file = cache_dir.join("version_manifest.json");
            if cache_file.exists() {
                let metadata = fs::metadata(&cache_file).await?;
                let modified = metadata.modified()?;
                if modified
                    .elapsed()
                    .map(|e| e.as_secs() < 3600)
                    .unwrap_or(false)
                {
                    let content = fs::read_to_string(&cache_file).await?;
                    if let Ok(manifest) = serde_json::from_str::<VersionManifest>(&content) {
                        return Ok(manifest);
                    }
                }
            }
        }

        info!("Fetching version manifest from {}", VERSION_MANIFEST_URL);
        let manifest = retry_async(
            || async {
                self.client
                    .get(VERSION_MANIFEST_URL)
                    .send()
                    .await?
                    .json::<VersionManifest>()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            "Fetch version manifest",
        )
        .await?;

        if let Some(cache_dir) = &self.cache_dir {
            if !cache_dir.exists() {
                fs::create_dir_all(cache_dir).await?;
            }
            let cache_file = cache_dir.join("version_manifest.json");
            let content = serde_json::to_string_pretty(&manifest)?;
            fs::write(cache_file, content).await?;
        }

        Ok(manifest)
    }

    pub async fn download_server<F>(
        &self,
        version_id: &str,
        target_path: impl AsRef<Path>,
        on_progress: F,
    ) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let manifest = self.fetch_manifest().await?;
        let version_info = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow!("Version {} not found in manifest", version_id))?;

        info!("Fetching details for version {}", version_id);
        let detail = retry_async(
            || async {
                self.client
                    .get(&version_info.url)
                    .send()
                    .await?
                    .json::<VersionDetail>()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            &format!("Fetch version details for {}", version_id),
        )
        .await?;

        let server_download = detail.downloads.server;
        let total_size = server_download.size;
        let expected_sha1 = server_download.sha1;
        let target_path = target_path.as_ref();

        // 1. Check ArtifactStore first
        if let Some(ref store) = self.artifact_store {
            if store.exists(&expected_sha1, HashAlgorithm::Sha1).await {
                debug!("Version {} found in artifact store, provisioning...", version_id);
                store.provision(&expected_sha1, HashAlgorithm::Sha1, target_path).await?;
                on_progress(total_size, total_size);
                return Ok(());
            }
        }

        // 2. Not in store, download to a temporary file first
        info!(
            "Downloading server JAR for version {}: {} ({} bytes)",
            version_id, server_download.url, total_size
        );

        let temp_dir = self.cache_dir.as_ref()
            .map(|p| p.join("temp"))
            .unwrap_or_else(|| std::env::temp_dir());
        
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).await?;
        }
        
        let temp_file_path = temp_dir.join(format!("mc_server_{}_{}.jar.tmp", version_id, expected_sha1));

        retry_async(
            || async {
                let response = self.client.get(&server_download.url).send().await?;
                on_progress(0, total_size);

                let mut file = fs::File::create(&temp_file_path).await?;
                let mut hasher = Sha1::new();
                let mut downloaded: u64 = 0;

                let mut stream = response.bytes_stream();
                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    file.write_all(&chunk).await?;
                    hasher.update(&chunk);
                    downloaded += chunk.len() as u64;
                    on_progress(downloaded, total_size);
                }

                file.flush().await?;

                let actual_sha1 = format!("{:x}", hasher.finalize());
                if actual_sha1 != expected_sha1 {
                    let _ = fs::remove_file(&temp_file_path).await;
                    return Err(anyhow!(
                        "SHA1 mismatch! Expected: {}, Got: {}",
                        expected_sha1,
                        actual_sha1
                    ));
                }
                Ok(())
            },
            3,
            Duration::from_secs(2),
            &format!("Download server JAR for {}", version_id),
        )
        .await?;

        // 3. Add to ArtifactStore and then provision (or move if store not available)
        if let Some(ref store) = self.artifact_store {
            store.add_artifact(&temp_file_path, &expected_sha1, HashAlgorithm::Sha1).await?;
            store.provision(&expected_sha1, HashAlgorithm::Sha1, target_path).await?;
            let _ = fs::remove_file(&temp_file_path).await;
        } else {
            // Fallback: move temp file to target path
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::rename(&temp_file_path, target_path).await
                .with_context(|| format!("Failed to move temp file to {:?}", target_path))?;
        }

        info!(
            "Successfully downloaded and verified server JAR for version {}",
            version_id
        );
        Ok(())
    }
}
