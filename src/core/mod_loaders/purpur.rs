use serde::Deserialize;
use anyhow::Result;
use super::ModLoaderClient;

#[derive(Debug, Deserialize)]
pub struct PurpurVersions {
    pub builds: PurpurBuilds,
}

#[derive(Debug, Deserialize)]
pub struct PurpurBuilds {
    pub all: Vec<String>,
}

impl ModLoaderClient {
    pub async fn get_purpur_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        let cache_key = format!("purpur_versions_{}", mc_version);
        if let Ok(Some(cached)) = self.cache.get::<Vec<String>>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("https://api.purpurmc.org/v2/purpur/{}", mc_version);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let purpur_versions: PurpurVersions = response.json().await?;
        let mut versions = purpur_versions.builds.all;
        
        versions.reverse(); // Newest builds first
        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }

    pub async fn download_purpur<F>(&self, mc_version: &str, build: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        let url = format!("https://api.purpurmc.org/v2/purpur/{}/{}/download", mc_version, build);
        self.download_with_progress(&url, target_path, on_progress).await
    }
}
