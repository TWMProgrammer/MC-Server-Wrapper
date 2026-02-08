use anyhow::{Result, anyhow};
use super::HangarClient;
use crate::plugins::types::{ProjectVersion, ProjectFile};

impl HangarClient {
    pub async fn get_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let cache_key = format!("hangar_versions_{}_v:{:?}_lo:{:?}", project_id, game_version, loader);
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let project_id = project_id.to_string();

        self.cache.fetch_with_cache(
            cache_key,
            std::time::Duration::from_secs(3600),
            move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let project_id = project_id.clone();
                async move {
                    let url = format!("{}/projects/{}/versions", base_url, project_id);
                    let response: serde_json::Value = client.get(&url).send().await?.json().await?;
                    
                    let result = response["result"].as_array().ok_or_else(|| anyhow!("Invalid response from Hangar"))?;
                    
                    let mut versions = Vec::new();
                    for v in result {
                        let version_name = v["name"].as_str().unwrap_or_default().to_string();
                        
                        let mut files = Vec::new();
                        
                        // Hangar can have multiple platforms (PAPER, WATERFALL, VELOCITY)
                        // We'll look for PAPER first as it's the most common
                        if let Some(paper_downloads) = v["downloads"].get("PAPER") {
                            let download_url = paper_downloads["downloadUrl"].as_str();
                            let external_url = paper_downloads["externalUrl"].as_str();
                            
                            if let Some(url) = download_url.or(external_url) {
                                files.push(ProjectFile {
                                    url: url.to_string(),
                                    filename: format!("{}-{}.jar", project_id.split('/').last().unwrap_or("plugin"), version_name),
                                    primary: true,
                                    size: 0, // Hangar doesn't always provide size in this view
                                    sha1: None,
                                });
                            }
                        }

                        if files.is_empty() {
                            continue;
                        }

                        let mut loaders = Vec::new();
                        if v["downloads"].get("PAPER").is_some() { loaders.push("Paper".to_string()); }
                        if v["downloads"].get("WATERFALL").is_some() { loaders.push("Waterfall".to_string()); }
                        if v["downloads"].get("VELOCITY").is_some() { loaders.push("Velocity".to_string()); }

                        versions.push(ProjectVersion {
                            id: version_name.clone(),
                            project_id: project_id.clone(),
                            version_number: version_name,
                            files,
                            loaders,
                            game_versions: vec![], // Hangar versions are tied to platforms, game versions are separate
                        });
                    }

                    Ok(versions)
                }
            }
        ).await
    }
}
