use anyhow::Result;
use super::client::ModLoaderClient;
use super::types::ModLoader;

impl ModLoaderClient {
    pub async fn get_available_loaders(
        &self,
        mc_version: &str,
        server_type: Option<&str>,
    ) -> Result<Vec<ModLoader>> {
        let mut loaders = Vec::new();

        // If we know the server type, we can skip the bedrock check if it's not bedrock
        let is_bedrock = if let Some(t) = server_type {
            t.to_lowercase() == "bedrock"
        } else {
            let bedrock_manifest = self.get_bedrock_versions().await.unwrap_or_else(|_| {
                crate::downloader::VersionManifest {
                    latest: crate::downloader::LatestVersions {
                        release: "".to_string(),
                        snapshot: "".to_string(),
                    },
                    versions: vec![],
                }
            });
            bedrock_manifest.versions.iter().any(|v| v.id == mc_version)
        };

        if is_bedrock {
            // This is a Bedrock version, don't return Java loaders
            return Ok(vec![]);
        }

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

        // Proxies (Velocity/BungeeCord)
        if let Ok(versions) = self.get_velocity_versions().await {
            loaders.push(ModLoader {
                name: "Velocity".to_string(),
                versions,
            });
        }

        if let Ok(versions) = self.get_bungeecord_versions().await {
            loaders.push(ModLoader {
                name: "BungeeCord".to_string(),
                versions,
            });
        }

        Ok(loaders)
    }
}
