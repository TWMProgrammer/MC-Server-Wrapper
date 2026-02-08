use super::HangarClient;
use crate::plugins::types::ProjectVersion;
use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tracing::info;

use crate::artifacts::HashAlgorithm;
use crate::utils::{DownloadOptions, download_with_resumption};

impl HangarClient {
    pub async fn download_version(
        &self,
        version: &ProjectVersion,
        target_dir: impl AsRef<Path>,
    ) -> Result<String> {
        let file = version
            .files
            .iter()
            .find(|f| f.primary)
            .or_else(|| version.files.first())
            .ok_or_else(|| anyhow!("No files found for version"))?;

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        // Hangar's downloadUrl might be a relative path or need a base URL
        let download_url = if file.url.starts_with('/') {
            format!("https://hangar.papermc.io{}", file.url)
        } else {
            file.url.clone()
        };

        let target_path = target_dir.as_ref().join(&file.filename);
        info!(
            "Downloading plugin from {}: {}",
            download_url, file.filename
        );

        download_with_resumption(
            &self.client,
            DownloadOptions {
                url: &download_url,
                target_path: &target_path,
                expected_hash: file
                    .sha1
                    .as_ref()
                    .map(|h| (h.as_str(), HashAlgorithm::Sha1)),
                total_size: Some(file.size),
            },
            |_, _| {},
        )
        .await?;

        Ok(file.filename.clone())
    }
}
