use anyhow::{Result, anyhow};
use super::CurseForgeClient;
use crate::mods::types::{ProjectVersion, ProjectFile, Dependency};

impl CurseForgeClient {
    pub async fn get_dependencies(&self, project_id: &str, _game_version: Option<&str>, _loader: Option<&str>) -> Result<Vec<crate::mods::types::ResolvedDependency>> {
        let cache_key = format!("curseforge_dependencies_{}_v:{:?}_lo:{:?}", project_id, _game_version, _loader);
        if let Ok(Some(cached)) = self.cache.get::<Vec<crate::mods::types::ResolvedDependency>>(&cache_key).await {
            return Ok(cached);
        }

        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        let url = format!("https://api.curseforge.com/v1/mods/{}", project_id);
        let response = self.client.get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let dependencies = response["data"]["dependencies"].as_array();
        if let Some(deps) = dependencies {
            let mut resolved_deps = Vec::new();
            for dep in deps {
                let relation_type = dep["relationType"].as_u64();
                
                // Mapping relation types:
                // 3: Required
                // 2: Optional
                // Others: Incompatible, Embedded, etc.
                let dep_type = match relation_type {
                    Some(3) => "required",
                    Some(2) => "optional",
                    _ => continue, // Skip others for now
                };

                if let Some(mod_id) = dep["modId"].as_u64() {
                    if let Ok(project) = self.get_project(&mod_id.to_string()).await {
                        resolved_deps.push(crate::mods::types::ResolvedDependency {
                            project,
                            dependency_type: dep_type.to_string(),
                        });
                    }
                }
            }
            let _ = self.cache.set(cache_key, resolved_deps.clone()).await;
            Ok(resolved_deps)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_versions(
        &self, 
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let cache_key = format!("curseforge_versions_{}_v:{:?}_lo:{:?}", project_id, game_version, loader);
        if let Ok(Some(cached)) = self.cache.get::<Vec<ProjectVersion>>(&cache_key).await {
            return Ok(cached);
        }

        let api_key = self.api_key.as_ref().ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        let url = format!("https://api.curseforge.com/v1/mods/{}/files", project_id);
        
        let mut query_params = Vec::new();
        if let Some(gv) = game_version {
            query_params.push(("gameVersion", gv.to_string()));
        }
        if let Some(l) = loader {
            let loader_type = match l.to_lowercase().as_str() {
                "forge" => "1",
                "fabric" => "4",
                "quilt" => "5",
                "neoforge" => "6",
                _ => "0",
            };
            if loader_type != "0" {
                query_params.push(("modLoaderType", loader_type.to_string()));
            }
        }

        let response = self.client.get(&url)
            .header("x-api-key", api_key)
            .query(&query_params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let data = response["data"].as_array().ok_or_else(|| anyhow!("Invalid response from CurseForge"))?;
        
        let versions: Vec<ProjectVersion> = data.iter().map(|v| ProjectVersion {
            id: v["id"].as_u64().unwrap_or(0).to_string(),
            project_id: project_id.to_string(),
            version_number: v["displayName"].as_str().unwrap_or_default().to_string(),
            files: vec![ProjectFile {
                url: v["downloadUrl"].as_str().unwrap_or_default().to_string(),
                filename: v["fileName"].as_str().unwrap_or_default().to_string(),
                primary: true,
                size: v["fileLength"].as_u64().unwrap_or(0),
            }],
            loaders: v["gameVersions"].as_array().map(|gv| {
                gv.iter().filter_map(|s: &serde_json::Value| s.as_str().map(|s| s.to_string()))
                  .filter(|s: &String| ["Forge", "Fabric", "Quilt", "NeoForge"].contains(&s.as_str()))
                  .collect()
            }).unwrap_or_default(),
            game_versions: v["gameVersions"].as_array().map(|gv| {
                gv.iter().filter_map(|s: &serde_json::Value| s.as_str().map(|s| s.to_string()))
                  .filter(|s: &String| s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
                  .collect()
            }).unwrap_or_default(),
            dependencies: v["dependencies"].as_array().map(|deps| {
                deps.iter().filter_map(|d| {
                    let relation_type = d["relationType"].as_u64();
                    if matches!(relation_type, Some(3) | Some(2)) {
                        Some(Dependency {
                            project_id: d["modId"].as_u64().map(|id| id.to_string()),
                            version_id: None,
                            dependency_type: if relation_type == Some(3) { "required".to_string() } else { "optional".to_string() },
                        })
                    } else {
                        None
                    }
                }).collect()
            }).unwrap_or_default(),
        }).collect();

        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }
}
