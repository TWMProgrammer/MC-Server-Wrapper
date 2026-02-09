use super::ModrinthClient;
use crate::artifacts::HashAlgorithm;
use crate::mods::types::ProjectVersion;
use crate::utils::{DownloadOptions, download_with_resumption};
use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tracing::info;

impl ModrinthClient {
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

        let target_path = target_dir.as_ref().join(&file.filename);
        info!(
            "Downloading mod from {}: {} ({} bytes)",
            file.url, file.filename, file.size
        );

        download_with_resumption(
            self.inner.cache.get_client(),
            DownloadOptions {
                url: &file.url,
                target_path: &target_path,
                expected_hash: file
                    .sha1
                    .as_ref()
                    .map(|h| (h.as_str(), HashAlgorithm::Sha1)),
                total_size: Some(file.size),
            },
            |_, _| {}, // Mod downloads don't seem to use progress reporting here yet
        )
        .await?;

        Ok(file.filename.clone())
    }
}
