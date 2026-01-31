use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLoader {
    pub name: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FabricLoaderVersion {
    pub loader: FabricLoader,
}

#[derive(Debug, Deserialize)]
struct FabricLoader {
    pub version: String,
    pub stable: bool,
}

pub struct ModLoaderClient {
    client: reqwest::Client,
}

impl ModLoaderClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_fabric_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let loaders: Vec<FabricLoaderVersion> = response.json().await?;
        let versions = loaders.into_iter()
            .map(|l| l.loader.version)
            .collect();
        
        Ok(versions)
    }

    pub async fn get_forge_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        // Forge is more complex, for now let's just return an empty list or a mock
        // Real implementation would parse maven-metadata.xml or a similar source
        Ok(vec![])
    }

    pub async fn get_available_loaders(&self, mc_version: &str) -> Result<Vec<ModLoader>> {
        let mut loaders = Vec::new();

        // Fabric
        let fabric_versions = self.get_fabric_versions(mc_version).await?;
        if !fabric_versions.is_empty() {
            loaders.push(ModLoader {
                name: "Fabric".to_string(),
                versions: fabric_versions,
            });
        }

        // Forge (Stub)
        let forge_versions = self.get_forge_versions(mc_version).await?;
        if !forge_versions.is_empty() {
            loaders.push(ModLoader {
                name: "Forge".to_string(),
                versions: forge_versions,
            });
        }

        Ok(loaders)
    }
}
