use crate::utils::retry_async;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use sha1::{Sha1, Digest};
use futures_util::StreamExt;
use tracing::info;
use chrono::{DateTime, Utc};

const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionInfo {
    pub id: String,
    pub r#type: String,
    pub url: String,
    #[serde(rename = "releaseTime")]
    pub release_date: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct VersionDetail {
    pub downloads: Downloads,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub server: DownloadInfo,
}

#[derive(Debug, Deserialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

pub struct VersionDownloader {
    client: reqwest::Client,
    cache_dir: Option<PathBuf>,
}

impl VersionDownloader {
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache_dir,
        }
    }

    pub async fn fetch_manifest(&self) -> Result<VersionManifest> {
        if let Some(cache_dir) = &self.cache_dir {
            let cache_file = cache_dir.join("version_manifest.json");
            if cache_file.exists() {
                // Check if cache is fresh (e.g., less than 1 hour old)
                let metadata = fs::metadata(&cache_file).await?;
                let modified = metadata.modified()?;
                if modified.elapsed().map(|e| e.as_secs() < 3600).unwrap_or(false) {
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
                self.client.get(VERSION_MANIFEST_URL)
                    .send()
                    .await?
                    .json::<VersionManifest>()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            "Fetch version manifest"
        ).await?;

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

    pub async fn download_server<F>(&self, version_id: &str, target_path: impl AsRef<Path>, on_progress: F) -> Result<()> 
    where 
        F: Fn(u64, u64) + Send + Sync + 'static 
    {
        let manifest = self.fetch_manifest().await?;
        let version_info = manifest.versions.iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow!("Version {} not found in manifest", version_id))?;

        info!("Fetching details for version {}", version_id);
        let detail = retry_async(
            || async {
                self.client.get(&version_info.url)
                    .send()
                    .await?
                    .json::<VersionDetail>()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            &format!("Fetch version details for {}", version_id)
        ).await?;

        let server_download = detail.downloads.server;
        let total_size = server_download.size;
        info!("Downloading server JAR for version {}: {} ({} bytes)", version_id, server_download.url, total_size);

        retry_async(
            || async {
                let response = self.client.get(&server_download.url).send().await?;
                let mut file = fs::File::create(&target_path).await?;
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
                if actual_sha1 != server_download.sha1 {
                    fs::remove_file(&target_path).await?;
                    return Err(anyhow!("SHA1 mismatch! Expected: {}, Got: {}", server_download.sha1, actual_sha1));
                }
                Ok(())
            },
            3,
            Duration::from_secs(2),
            &format!("Download server JAR for {}", version_id)
        ).await?;

        info!("Successfully downloaded and verified server JAR for version {}", version_id);
        Ok(())
    }
}

