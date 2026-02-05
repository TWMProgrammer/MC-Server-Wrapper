use anyhow::{Result, anyhow};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;
use crate::utils::retry_async;
use super::ModrinthClient;
use crate::mods::types::ProjectVersion;

impl ModrinthClient {
    pub async fn download_version(&self, version: &ProjectVersion, target_dir: impl AsRef<Path>) -> Result<String> {
        let file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first())
            .ok_or_else(|| anyhow!("No files found for version"))?;

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let target_path = target_dir.as_ref().join(&file.filename);
        info!("Downloading mod from {}: {} ({} bytes)", file.url, file.filename, file.size);

        let target_path_clone = target_path.clone();
        retry_async(
            || async {
                let response = self.client.get(&file.url).send().await?;
                let mut f = fs::File::create(&target_path_clone).await?;

                let mut stream = response.bytes_stream();
                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    f.write_all(&chunk).await?;
                }

                f.flush().await?;
                Ok(())
            },
            3,
            Duration::from_secs(2),
            &format!("Download mod: {}", file.filename)
        ).await?;

        Ok(file.filename.clone())
    }
}
