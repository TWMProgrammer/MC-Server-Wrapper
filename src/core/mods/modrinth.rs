use crate::utils::retry_async;
use anyhow::{Result, anyhow};
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tracing::info;
use super::types::{Project, ProjectVersion, ModProvider, SearchOptions, SortOrder, ResolvedDependency};

pub struct ModrinthClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ModrinthClient {
    pub fn new() -> Self {
        Self::with_base_url("https://api.modrinth.com/v2".to_string())
    }

    pub fn with_base_url(base_url: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("mc-server-wrapper/0.1.0")
                .build()
                .expect("Failed to create reqwest client"),
            base_url,
        }
    }

    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let mut url = format!("{}/search?query={}", self.base_url, urlencoding::encode(&options.query));

        let mut and_groups: Vec<Vec<String>> = Vec::new();

        if let Some(facets) = &options.facets {
            if !facets.is_empty() {
                for f in facets {
                    and_groups.push(vec![f.clone()]);
                }
            }
        }

        // Add project_type:mod if not already present
        let mut has_type = false;
        for group in &and_groups {
            if group.iter().any(|f| f.starts_with("project_type:")) {
                has_type = true;
                break;
            }
        }
        if !has_type {
            and_groups.push(vec!["project_type:mod".to_string()]);
        }

        if let Some(version) = &options.game_version {
            if !version.is_empty() {
                and_groups.push(vec![format!("versions:{}", version)]);
            }
        }

        if let Some(loader) = &options.loader {
            if !loader.is_empty() {
                and_groups.push(vec![format!("categories:{}", loader.to_lowercase())]);
            }
        }

        if !and_groups.is_empty() {
            let facets_json = serde_json::to_string(&and_groups)?;
            url.push_str(&format!("&facets={}", urlencoding::encode(&facets_json)));
        }

        if let Some(sort) = options.sort {
            let index = match sort {
                SortOrder::Relevance => "relevance",
                SortOrder::Downloads => "downloads",
                SortOrder::Follows => "follows",
                SortOrder::Newest => "newest",
                SortOrder::Updated => "updated",
            };
            url.push_str(&format!("&index={}", index));
        }

        if let Some(offset) = options.offset {
            url.push_str(&format!("&offset={}", offset));
        }

        if let Some(limit) = options.limit {
            url.push_str(&format!("&limit={}", limit));
        }

        let response_text: String = retry_async(
            || async {
                let res = self.client.get(&url)
                    .send()
                    .await?;
                let text = res.text()
                    .await
                    .map_err(|e| anyhow!(e))?;
                Ok(text)
            },
            3,
            Duration::from_secs(2),
            &format!("Modrinth search: {}", options.query)
        ).await?;

        let response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Modrinth response: {}. Body: {}", e, response_text))?;
        
        let hits = response["hits"].as_array().ok_or_else(|| anyhow!("Invalid response from Modrinth: missing 'hits' field"))?;
        
        let projects = hits.iter().map(|h| Project {
            id: h["project_id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s: &str| s.to_string()),
            author: h["author"].as_str().unwrap_or_default().to_string(),
            provider: ModProvider::Modrinth,
            categories: h["categories"].as_array().map(|cats: &Vec<serde_json::Value>| {
                cats.iter().filter_map(|c: &serde_json::Value| c.as_str().map(|s: &str| s.to_string())).collect()
            }),
        }).collect();

        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let url = format!("{}/project/{}", self.base_url, id);
        let h = retry_async(
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
            &format!("Get Modrinth project: {}", id)
        ).await?;
        
        Ok(Project {
            id: h["id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s: &str| s.to_string()),
            author: String::new(), // Author is in a separate field
            provider: ModProvider::Modrinth,
            categories: h["categories"].as_array().map(|cats: &Vec<serde_json::Value>| {
                cats.iter().filter_map(|c: &serde_json::Value| c.as_str().map(|s: &str| s.to_string())).collect()
            }),
        })
    }

    pub async fn get_dependencies(&self, project_id: &str, game_version: Option<&str>, loader: Option<&str>) -> Result<Vec<ResolvedDependency>> {
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

        Ok(resolved_deps)
    }

    pub async fn get_versions(
        &self, 
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
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
        Ok(versions)
    }

    pub async fn download_version(&self, version: &ProjectVersion, target_dir: impl AsRef<Path>) -> Result<String> {
        let file = version.files.iter().find(|f| f.primary).or_else(|| version.files.first())
            .ok_or_else(|| anyhow!("No files found for version"))?;

        if !target_dir.as_ref().exists() {
            fs::create_dir_all(&target_dir).await?;
        }

        let target_path = target_dir.as_ref().join(&file.filename);
        info!("Downloading mod from {}: {} ({} bytes)", file.url, file.filename, file.size);

        let target_path_clone = target_path.clone();
        retry_async(
            || async {
                let response = self.client.get(&file.url).send().await?;
                let mut f = fs::File::create(&target_path_clone).await?;

                let mut stream = response.bytes_stream();
                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    f.write_all(&chunk).await?;
                }

                f.flush().await?;
                Ok(())
            },
            3,
            Duration::from_secs(2),
            &format!("Download mod: {}", file.filename)
        ).await?;

        Ok(file.filename.clone())
    }
}