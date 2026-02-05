use anyhow::{Result, anyhow};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use super::CurseForgeClient;

impl CurseForgeClient {
    pub async fn download_mod(&self, _mod_id: &str, _file_id: &str, _target_dir: impl AsRef<Path>) -> Result<String> {
        let _api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        
        // In CurseForge v1, we can get file info by file_id directly if we have the mod_id, 
        // but often we just need the download URL from the version info.
        // For simplicity, let's assume we get the version first.
        
        // If we only have file_id, we need to fetch it.
        // Actually, the v1 API requires modId. Let's assume we have it or use a different endpoint.
        
        Err(anyhow!("Use download_file instead with the URL from ProjectFile"))
    }

    pub async fn download_file(&self, url: &str, filename: &str, target_dir: impl AsRef<Path>) -> Result<String> {
        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let target_path = target_dir.as_ref().join(filename);
        let response = self.client.get(url).send().await?.error_for_status()?;
        
        let mut f = fs::File::create(&target_path).await?;
        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            f.write_all(&chunk).await?;
        }

        f.flush().await?;
        Ok(filename.to_string())
    }
}
