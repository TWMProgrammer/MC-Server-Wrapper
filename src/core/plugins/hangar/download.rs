use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;
use super::HangarClient;
use crate::plugins::types::ProjectVersion;

impl HangarClient {
    pub async fn download_version(&self, version: &ProjectVersion, target_dir: impl AsRef<Path>) -> Result<String> {
        let file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first())
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
        info!("Downloading plugin from {}: {}", download_url, file.filename);

        let response = self.client.get(&download_url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download plugin: HTTP {}", response.status()));
        }

        let mut f = fs::File::create(&target_path).await?;

        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            f.write_all(&chunk).await?;
        }

        f.flush().await?;
        Ok(file.filename.clone())
    }
}
