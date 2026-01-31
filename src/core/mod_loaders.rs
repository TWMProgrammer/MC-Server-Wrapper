use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::info;

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
}

#[derive(Debug, Deserialize)]
struct FabricInstallerVersion {
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct ForgePromotions {
    pub promos: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct PaperBuilds {
    pub builds: Vec<PaperBuildSummary>,
}

#[derive(Debug, Deserialize)]
struct PaperBuildSummary {
    pub build: u32,
}

#[derive(Debug, Deserialize)]
struct PaperBuildDetails {
    pub downloads: PaperDownloads,
}

#[derive(Debug, Deserialize)]
struct PaperDownloads {
    pub application: PaperDownload,
}

#[derive(Debug, Deserialize)]
struct PaperDownload {
    pub name: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
struct PurpurVersions {
    pub builds: PurpurBuilds,
}

#[derive(Debug, Deserialize)]
struct PurpurBuilds {
    pub all: Vec<String>,
}

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

pub struct ModLoaderClient {
    client: reqwest::Client,
}

impl ModLoaderClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    async fn download_with_progress<F>(&self, url: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut file = tokio::fs::File::create(target_path).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            on_progress(downloaded, total_size);
        }

        file.flush().await?;
        Ok(())
    }

    pub async fn get_fabric_versions(&self, mc_version: &str) -> Result<Vec<String>> {
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
        let versions = loaders.into_iter()
            .map(|l| l.loader.version)
            .collect();
        
        Ok(versions)
    }

    pub async fn get_fabric_installer_versions(&self) -> Result<Vec<String>> {
        let url = "https://meta.fabricmc.net/v2/versions/installer";
        let response = self.client.get(url).send().await?;
        let installers: Vec<FabricInstallerVersion> = response.json().await?;
        Ok(installers.into_iter().map(|i| i.version).collect())
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

    pub async fn get_neoforge_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        // NeoForge uses Maven metadata. For now, let's try to fetch it and parse with regex 
        // to avoid adding an XML dependency if possible, or just use a known API if it exists.
        // NeoForge doesn't have a simple "meta" API like Fabric/Quilt yet that is widely known/stable.
        // However, we can fetch from Maven: https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml
        
        let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let text = response.text().await?;
        let mut versions = Vec::new();

        // Simple regex to extract versions from <version> tags
        // Maven metadata looks like <versioning><versions><version>...</version></versions></versioning>
        // For NeoForge, version names are like "20.1.100" (for 1.20.1) or "21.0.1" (for 1.21)
        // NeoForge versions usually start with the major MC version (e.g., 20.x for 1.20.x)
        
        let mc_parts: Vec<&str> = mc_version.split('.').collect();
        if mc_parts.len() < 2 {
            return Ok(vec![]);
        }
        
        let major = mc_parts[1]; // "20" from "1.20.1"
        
        // NeoForge versioning: 
        // 1.20.1 -> versions start with 20.1
        // 1.21 -> versions start with 21.0
        // 1.21.1 -> versions start with 21.1
        let prefix = if mc_parts.len() > 2 {
            format!("{}.{}", major, mc_parts[2])
        } else {
            format!("{}.0", major)
        };

        let re = regex::Regex::new(r"<version>([^<]+)</version>")?;
        for cap in re.captures_iter(&text) {
            let ver = &cap[1];
            if ver.starts_with(&prefix) {
                versions.push(ver.to_string());
            }
        }

        // Sort versions numerically (descending)
        versions.sort_by(|a, b| {
            let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
            let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();
            b_parts.cmp(&a_parts)
        });

        Ok(versions)
    }

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

    pub async fn download_forge<F>(&self, mc_version: &str, forge_version: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        // Forge download URL pattern: https://maven.minecraftforge.net/net/minecraftforge/forge/{mc_version}-{forge_version}/forge-{mc_version}-{forge_version}-installer.jar
        let version_str = format!("{}-{}", mc_version, forge_version);
        let url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}/forge-{}-installer.jar", version_str, version_str);
        self.download_with_progress(&url, target_path, on_progress).await
    }

    pub async fn download_purpur<F>(&self, mc_version: &str, build: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        let url = format!("https://api.purpurmc.org/v2/purpur/{}/{}/download", mc_version, build);
        self.download_with_progress(&url, target_path, on_progress).await
    }

    pub async fn download_neoforge<F>(&self, neoforge_version: &str, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        // NeoForge download URL pattern: https://maven.neoforged.net/releases/net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar
        let url = format!("https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar", neoforge_version, neoforge_version);
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

    pub async fn download_loader<F>(&self, loader_name: &str, mc_version: &str, loader_version: Option<&str>, target_path: impl AsRef<std::path::Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64) + Send + Sync + 'static {
        match loader_name.to_lowercase().as_str() {
            "paper" => {
                let build = loader_version.ok_or_else(|| anyhow::anyhow!("Paper requires a build number"))?;
                self.download_paper(mc_version, build, target_path, on_progress).await
            },
            "fabric" => {
                let version = loader_version.ok_or_else(|| anyhow::anyhow!("Fabric requires a loader version"))?;
                self.download_fabric(mc_version, version, target_path, on_progress).await
            },
            "forge" => {
                let version = loader_version.ok_or_else(|| anyhow::anyhow!("Forge requires a version"))?;
                self.download_forge(mc_version, version, target_path, on_progress).await
            },
            "purpur" => {
                let build = loader_version.ok_or_else(|| anyhow::anyhow!("Purpur requires a build number"))?;
                self.download_purpur(mc_version, build, target_path, on_progress).await
            },
            "neoforge" => {
                let version = loader_version.ok_or_else(|| anyhow::anyhow!("NeoForge requires a version"))?;
                self.download_neoforge(version, target_path, on_progress).await
            },
            _ => Err(anyhow::anyhow!("Unsupported mod loader: {}", loader_name)),
        }
    }
    pub async fn get_purpur_versions(&self, mc_version: &str) -> Result<Vec<String>> {
        let url = format!("https://api.purpurmc.org/v2/purpur/{}", mc_version);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let purpur_versions: PurpurVersions = response.json().await?;
        let mut versions = purpur_versions.builds.all;
        
        versions.reverse(); // Newest builds first
        Ok(versions)
    }

    pub async fn get_available_loaders(&self, mc_version: &str) -> Result<Vec<ModLoader>> {
        let mut loaders = Vec::new();

        // Fabric
        if let Ok(versions) = self.get_fabric_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Fabric".to_string(),
                    versions,
                });
            }
        }

        // Forge
        if let Ok(versions) = self.get_forge_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Forge".to_string(),
                    versions,
                });
            }
        }

        // NeoForge
        if let Ok(versions) = self.get_neoforge_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "NeoForge".to_string(),
                    versions,
                });
            }
        }

        // Paper
        if let Ok(versions) = self.get_paper_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Paper".to_string(),
                    versions,
                });
            }
        }

        // Purpur
        if let Ok(versions) = self.get_purpur_versions(mc_version).await {
            if !versions.is_empty() {
                loaders.push(ModLoader {
                    name: "Purpur".to_string(),
                    versions,
                });
            }
        }

        Ok(loaders)
    }
}