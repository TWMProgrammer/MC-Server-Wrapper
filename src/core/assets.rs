use crate::cache::CacheManager;
use crate::utils::retry_async;
use anyhow::{Context, Result};
use hex;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;

pub struct AssetManager {
    cache_dir: PathBuf,
    client: reqwest::Client,
    default_ttl: Duration,
}

impl AssetManager {
    pub fn new(cache_dir: PathBuf, _cache_manager: Arc<CacheManager>) -> Self {
        // Ensure cache directory exists
        if let Err(e) = std::fs::create_dir_all(&cache_dir) {
            tracing::error!(
                "Failed to create asset cache directory {:?}: {}",
                cache_dir,
                e
            );
        }

        Self {
            cache_dir,
            client: reqwest::Client::builder()
                .user_agent(concat!("mc-server-wrapper/", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            // Default assets TTL: 7 days
            default_ttl: Duration::from_secs(60 * 60 * 24 * 7),
        }
    }

    /// Gets an asset from cache or downloads it.
    /// Returns the local path to the cached asset.
    pub async fn get_asset(&self, url: &str) -> Result<PathBuf> {
        let url_hash = {
            let mut hasher = Sha256::new();
            hasher.update(url.as_bytes());
            hex::encode(hasher.finalize())
        };

        // Determine extension from URL if possible, default to .png
        // Strip query parameters first to avoid illegal characters in filenames on Windows
        let url_path = url.split('?').next().unwrap_or(url);
        let extension = url_path.split('.').last().unwrap_or("png");

        // Sanitize extension: only allow alphanumeric characters and keep it short
        let extension = extension
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>();

        let extension = if extension.is_empty() || extension.len() > 5 {
            "png".to_string()
        } else {
            extension
        };

        let file_name = format!("{}.{}", url_hash, extension);
        let target_path = self.cache_dir.join(&file_name);

        // Check if file exists and is not too old
        if target_path.exists() {
            if let Ok(metadata) = fs::metadata(&target_path).await {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed < self.default_ttl {
                            return Ok(target_path);
                        }
                    }
                }
            }
        }

        // Download the asset
        tracing::debug!("Downloading asset: {} -> {:?}", url, target_path);

        let client = self.client.clone();
        let url_owned = url.to_string();

        let data = retry_async(
            || async {
                let response = client.get(&url_owned).send().await?;
                if !response.status().is_success() {
                    let err_msg = format!(
                        "Failed to download asset: {} (Status: {})",
                        url_owned,
                        response.status()
                    );
                    tracing::error!("{}", err_msg);
                    return Err(anyhow::anyhow!(err_msg));
                }
                let bytes = response.bytes().await?;
                tracing::debug!(
                    "Successfully downloaded {} bytes from {}",
                    bytes.len(),
                    url_owned
                );
                Ok(bytes)
            },
            3,
            Duration::from_secs(2),
            &format!("Download asset: {}", url),
        )
        .await?;

        fs::write(&target_path, data)
            .await
            .with_context(|| format!("Failed to write asset to cache at {:?}", target_path))?;

        tracing::info!("Cached asset: {} -> {:?}", url, target_path);

        Ok(target_path)
    }

    /// Specifically for player heads using mc-heads.net which supports both UUIDs and usernames
    pub async fn get_player_head(&self, identifier: &str) -> Result<PathBuf> {
        let url = format!("https://mc-heads.net/avatar/{}/64", identifier);
        self.get_asset(&url).await
    }

    /// Cleans up assets older than the specified duration.
    pub async fn cleanup_assets(&self, max_age: Duration) -> Result<u64> {
        let mut count = 0;
        let mut entries = fs::read_dir(&self.cache_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = entry.metadata().await?;
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > max_age {
                            fs::remove_file(path).await?;
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Gets statistics about the asset cache.
    pub async fn get_stats(&self) -> Result<AssetCacheStats> {
        let mut count = 0;
        let mut total_size = 0;
        let mut entries = fs::read_dir(&self.cache_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = entry.metadata().await?;
                count += 1;
                total_size += metadata.len();
            }
        }

        Ok(AssetCacheStats { count, total_size })
    }
}

#[derive(serde::Serialize)]
pub struct AssetCacheStats {
    pub count: u64,
    pub total_size: u64,
}
