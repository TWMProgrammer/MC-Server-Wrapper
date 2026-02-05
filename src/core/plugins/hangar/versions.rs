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
        if let Ok(Some(cached)) = self.cache.get::<Vec<ProjectVersion>>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("{}/projects/{}/versions", self.base_url, project_id);
        let response: serde_json::Value = self.client.get(&url).send().await?.json().await?;
        
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
                        filename: format!("{}-{}.jar", project_id.replace('/', "-"), version_name),
                        primary: true,
                        size: 0, // Hangar doesn't always provide size in this view
                    });
                }
            }

            if files.is_empty() {
                // Try other platforms if PAPER is not found
                for (_platform, platform_downloads) in v["downloads"].as_object().unwrap() {
                    let download_url = platform_downloads["downloadUrl"].as_str();
                    let external_url = platform_downloads["externalUrl"].as_str();
                    
                    if let Some(url) = download_url.or(external_url) {
                        files.push(ProjectFile {
                            url: url.to_string(),
                            filename: format!("{}-{}.jar", project_id.replace('/', "-"), version_name),
                            primary: true,
                            size: 0,
                        });
                        break; 
                    }
                }
            }

            if !files.is_empty() {
                versions.push(ProjectVersion {
                    id: version_name.clone(),
                    project_id: project_id.to_string(),
                    version_number: version_name,
                    files,
                    loaders: vec!["paper".to_string()], // Hangar is primarily for Paper
                    game_versions: vec![], // Hangar has complex game version mapping, leaving empty for now
                });
            }
        }

        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }
}
