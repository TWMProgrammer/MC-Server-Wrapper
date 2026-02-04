use serde::Deserialize;
use anyhow::Result;
use tracing::info;
use super::ModLoaderClient;

#[derive(Debug, Deserialize)]
pub struct FabricLoaderVersion {
    pub loader: FabricLoader,
}

#[derive(Debug, Deserialize)]
pub struct FabricLoader {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct FabricInstallerVersion {
    pub version: String,
}

impl ModLoaderClient {
    pub async fn get_fabric_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        let cache_key = format!("fabric_versions_{}", mc_version);
        if let Ok(Some(cached)) = self.cache.get::<Vec<String>>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                info!("No Fabric versions found for Minecraft version {}", mc_version);
                return Ok(vec![]);
            }
            return Err(anyhow::anyhow!("Fabric Meta API returned error: {}", response.status()));
        }

        let loaders: Vec<FabricLoaderVersion> = response.json().await?;
        let versions: Vec<String> = loaders.into_iter()
            .map(|l| l.loader.version)
            .collect();
        
        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }

    pub async fn get_fabric_installer_versions(&self) -> Result<Vec<String>> {
        let cache_key = "fabric_installer_versions".to_string();
        if let Ok(Some(cached)) = self.cache.get::<Vec<String>>(&cache_key).await {
            return Ok(cached);
        }

        let url = "https://meta.fabricmc.net/v2/versions/installer";
        let response = self.client.get(url).send().await?;
        let installers: Vec<FabricInstallerVersion> = response.json().await?;
        let versions: Vec<String> = installers.into_iter().map(|i| i.version).collect();
        
        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }

    pub async fn download_fabric_installer<F>(&self, installer_version: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        let url = format!("https://maven.fabricmc.net/net/fabricmc/fabric-installer/{}/fabric-installer-{}.jar", installer_version, installer_version);
        self.download_with_progress(&url, target_path, on_progress).await
    }

    pub async fn download_fabric<F>(&self, mc_version: &str, loader_version: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        // We use the server-jp-launcher from Fabric Meta
        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/server/jar", mc_version, loader_version);
        self.download_with_progress(&url, target_path, on_progress).await
    }
}
