use anyhow::{Result, anyhow};
use std::time::Duration;
use crate::utils::retry_async;
use super::ModrinthClient;
use crate::mods::types::{ProjectVersion, ResolvedDependency, Project, ModProvider};

impl ModrinthClient {
    pub async fn get_dependencies(&self, project_id: &str, game_version: Option<&str>, loader: Option<&str>) -> Result<Vec<ResolvedDependency>> {
        let cache_key = format!("modrinth_mods_deps_{}_v:{:?}_lo:{:?}", project_id, game_version, loader);
        if let Ok(Some(cached)) = self.cache.get::<Vec<ResolvedDependency>>(&cache_key).await {
            return Ok(cached);
        }

        // If we have version/loader, find the best version and its specific dependencies
        if let (Some(gv), Some(l)) = (game_version, loader) {
            let versions = self.get_versions(project_id, Some(gv), Some(l)).await?;
            let l_lower = l.to_lowercase();
            
            let best_version = versions.into_iter().find(|v| {
                v.game_versions.contains(&gv.to_string()) && 
                v.loaders.iter().any(|vl| vl.to_lowercase() == l_lower)
            });

            if let Some(version) = best_version {
                let mut resolved = Vec::new();
                for dep in version.dependencies {
                    if let Some(dep_project_id) = dep.project_id {
                        if dep.dependency_type == "required" || dep.dependency_type == "optional" {
                            if let Ok(project) = self.get_project(&dep_project_id).await {
                                resolved.push(ResolvedDependency {
                                    project,
                                    dependency_type: dep.dependency_type,
                                });
                            }
                        }
                    }
                }
                let _ = self.cache.set(cache_key, resolved.clone()).await;
                return Ok(resolved);
            }
        }

        // Fallback to project-wide dependencies if no specific version found or version/loader not provided
        let url = format!("{}/project/{}/dependencies", self.base_url, project_id);
        let response = retry_async(
            || async {
                self.client.get(&url)
                    .send()
                    .await?
                    .json::<serde_json::Value>()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            &format!("Get Modrinth dependencies: {}", project_id)
        ).await?;
        
        let projects_json = response["projects"].as_array().ok_or_else(|| anyhow!("Invalid dependencies response"))?;
        let versions_json = response["versions"].as_array().ok_or_else(|| anyhow!("Invalid dependencies response"))?;
        
        let mut resolved_deps = Vec::new();
        
        for h in projects_json {
            let project_type = h["project_type"].as_str().unwrap_or_default();
            if project_type != "mod" {
                continue;
            }

            let id = h["id"].as_str().unwrap_or_default().to_string();
            
            let dependency_type = versions_json.iter()
                .find(|v| v["project_id"].as_str() == Some(&id))
                .and_then(|v| v["dependency_type"].as_str())
                .unwrap_or("required")
                .to_string();

            resolved_deps.push(ResolvedDependency {
                project: Project {
                    id,
                    slug: h["slug"].as_str().unwrap_or_default().to_string(),
                    title: h["title"].as_str().unwrap_or_default().to_string(),
                    description: h["description"].as_str().unwrap_or_default().to_string(),
                    downloads: h["downloads"].as_u64().unwrap_or(0),
                    icon_url: h["icon_url"].as_str().map(|s: &str| s.to_string()),
                    author: String::new(),
                    provider: ModProvider::Modrinth,
                    categories: h["categories"].as_array().map(|cats: &Vec<serde_json::Value>| {
                        cats.iter().filter_map(|c: &serde_json::Value| c.as_str().map(|s: &str| s.to_string())).collect()
                    }),
                },
                dependency_type,
            });
        }

        let _ = self.cache.set(cache_key, resolved_deps.clone()).await;
        Ok(resolved_deps)
    }

    pub async fn get_versions(
        &self, 
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let cache_key = format!("modrinth_mods_versions_{}_v:{:?}_lo:{:?}", project_id, game_version, loader);
        if let Ok(Some(cached)) = self.cache.get::<Vec<ProjectVersion>>(&cache_key).await {
            return Ok(cached);
        }

        let mut url = format!("{}/project/{}/version", self.base_url, project_id);
        
        let mut query_params = Vec::new();
        if let Some(gv) = game_version {
            query_params.push(format!("game_versions=[\"{}\"]", gv));
        }
        if let Some(l) = loader {
            query_params.push(format!("loaders=[\"{}\"]", l.to_lowercase()));
        }

        if !query_params.is_empty() {
            url.push_str("?");
            url.push_str(&query_params.join("&"));
        }

        let versions = retry_async(
            || async {
                self.client.get(&url)
                    .send()
                    .await?
                    .json::<Vec<ProjectVersion>>()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            &format!("Get Modrinth versions: {}", project_id)
        ).await?;
        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }
}
