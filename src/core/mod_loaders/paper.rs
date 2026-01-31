use serde::Deserialize;
use anyhow::Result;
use super::ModLoaderClient;

#[derive(Debug, Deserialize)]
pub struct PaperBuilds {
    pub builds: Vec<PaperBuildSummary>,
}

#[derive(Debug, Deserialize)]
pub struct PaperBuildSummary {
    pub build: u32,
}

#[derive(Debug, Deserialize)]
pub struct PaperBuildDetails {
    pub downloads: PaperDownloads,
}

#[derive(Debug, Deserialize)]
pub struct PaperDownloads {
    pub application: PaperDownload,
}

#[derive(Debug, Deserialize)]
pub struct PaperDownload {
    pub name: String,
    pub sha256: String,
}

impl ModLoaderClient {
    pub async fn get_paper_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        let url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds", mc_version);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let paper_builds: PaperBuilds = response.json().await?;
        let mut versions: Vec<String> = paper_builds.builds.into_iter()
            .map(|b| b.build.to_string())
            .collect();
        
        versions.reverse(); // Newest builds first
        Ok(versions)
    }

    pub async fn download_paper<F>(&self, mc_version: &str, build: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        let url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}", mc_version, build);
        let response = self.client.get(&url).send().await?;
        let build_info: PaperBuildDetails = response.json().await?;
        
        let download_name = build_info.downloads.application.name;
        let download_url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/{}", mc_version, build, download_name);
        
        self.download_with_progress(&download_url, &target_path, on_progress).await?;

        // Verify SHA256
        use sha2::{Sha256, Digest};
        let bytes = tokio::fs::read(&target_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let actual_sha256 = format!("{:x}", hasher.finalize());
        
        if actual_sha256 != build_info.downloads.application.sha256 {
            tokio::fs::remove_file(&target_path).await?;
            return Err(anyhow::anyhow!("SHA256 mismatch for Paper download! Expected: {}, Got: {}", build_info.downloads.application.sha256, actual_sha256));
        }

        Ok(())
    }
}
