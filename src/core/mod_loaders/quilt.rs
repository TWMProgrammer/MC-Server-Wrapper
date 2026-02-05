use serde::Deserialize;
use anyhow::{Result, anyhow};
use tracing::info;
use super::ModLoaderClient;

#[derive(Debug, Deserialize)]
pub struct QuiltLoaderVersion {
    pub loader: QuiltLoader,
}

#[derive(Debug, Deserialize)]
pub struct QuiltLoader {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct QuiltInstallerVersion {
    pub version: String,
}

impl ModLoaderClient {
    pub async fn get_quilt_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        let cache_key = format!("quilt_versions_{}", mc_version);
        if let Ok(Some(cached)) = self.cache.get::<Vec<String>>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                info!("No Quilt versions found for Minecraft version {}", mc_version);
                return Ok(vec![]);
            }
            return Err(anyhow!("Quilt Meta API returned error: {}", response.status()));
        }

        let loaders: Vec<QuiltLoaderVersion> = response.json().await?;
        let versions: Vec<String> = loaders.into_iter()
            .map(|l| l.loader.version)
            .collect();
        
        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }

    pub async fn get_quilt_installer_versions(&self) -> Result<Vec<String>> {
        let cache_key = "quilt_installer_versions".to_string();
        if let Ok(Some(cached)) = self.cache.get::<Vec<String>>(&cache_key).await {
            return Ok(cached);
        }

        let url = "https://meta.quiltmc.org/v3/versions/installer";
        let response = self.client.get(url).send().await?;
        let installers: Vec<QuiltInstallerVersion> = response.json().await?;
        let versions: Vec<String> = installers.into_iter().map(|i| i.version).collect();
        
        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }

    pub async fn download_quilt_installer<F>(&self, installer_version: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        let url = format!("https://maven.quiltmc.org/repository/release/org/quiltmc/quilt-installer/{}/quilt-installer-{}.jar", installer_version, installer_version);
        self.download_with_progress(&url, target_path, on_progress).await
    }
}
