use serde::Deserialize;
use anyhow::Result;
use super::ModLoaderClient;

#[derive(Debug, Deserialize)]
pub struct ForgePromotions {
    pub promos: std::collections::HashMap<String, String>,
}

impl ModLoaderClient {
    pub async fn get_forge_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let promotions: ForgePromotions = response.json().await?;
        let mut versions = Vec::new();

        // Find versions matching mc_version
        // Keys are like "1.20.1-latest" or "1.20.1-recommended"
        for (key, val) in promotions.promos {
            if key.starts_with(mc_version) {
                versions.push(val);
            }
        }

        // Remove duplicates and sort (though promos are limited)
        versions.sort();
        versions.dedup();
        versions.reverse(); // Newest first

        Ok(versions)
    }

    pub async fn download_forge<F>(&self, mc_version: &str, forge_version: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        // Forge download URL pattern: https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar
        let version_str = format!("{}-{}", mc_version, forge_version);
        let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar", version_str, version_str);
        self.download_with_progress(&url, target_path, on_progress).await
    }

    pub fn is_modern_forge(&self, mc_version: &str) -> bool {
        // Forge 1.17+ is considered modern and uses the run.bat/run.sh system
        if let Some(version) = mc_version.split('.').nth(1) {
            if let Ok(minor) = version.parse::<u32>() {
                return minor >= 17;
            }
        }
        false
    }
}
