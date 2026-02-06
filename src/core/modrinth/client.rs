use super::types::*;
use crate::cache::CacheManager;
use crate::utils::retry_async;
use anyhow::{Result, anyhow};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;

pub struct ModrinthClient {
    pub client: Client,
    pub base_url: String,
    pub cache: Arc<CacheManager>,
}

impl ModrinthClient {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self::with_base_url("https://api.modrinth.com/v2".to_string(), cache)
    }

    pub fn with_base_url(base_url: String, cache: Arc<CacheManager>) -> Self {
        Self {
            client: cache.get_client().clone(),
            base_url,
            cache,
        }
    }

    pub async fn search(&self, options: &ModrinthSearchOptions) -> Result<Vec<ModrinthProject>> {
        let cache_key = format!("modrinth_common_search_{}", options.cache_key());
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let options = options.clone();

        self.cache
            .fetch_with_options(cache_key, Duration::from_secs(3600), false, move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let options = options.clone();
                async move {
                    let mut url = format!(
                        "{}/search?query={}",
                        base_url,
                        urlencoding::encode(&options.query)
                    );

                    let mut and_groups: Vec<Vec<String>> = Vec::new();

                    if let Some(facets) = &options.facets {
                        for f in facets {
                            and_groups.push(vec![f.clone()]);
                        }
                    }

                    if let Some(project_type) = options.project_type {
                        let type_str = match project_type {
                            ModrinthProjectType::Mod => "mod",
                            ModrinthProjectType::Plugin => "plugin",
                            ModrinthProjectType::ResourcePack => "resourcepack",
                            ModrinthProjectType::DataPack => "datapack",
                        };
                        and_groups.push(vec![format!("project_type:{}", type_str)]);
                    }

                    if let Some(version) = &options.game_version {
                        if !version.is_empty() {
                            and_groups.push(vec![format!("versions:{}", version)]);
                        }
                    }

                    if let Some(loader) = &options.loader {
                        if !loader.is_empty() {
                            // Modrinth uses 'categories' for loaders in search facets
                            and_groups.push(vec![format!("categories:{}", loader.to_lowercase())]);
                        }
                    }

                    if !and_groups.is_empty() {
                        let facets_json = serde_json::to_string(&and_groups)?;
                        url.push_str(&format!("&facets={}", urlencoding::encode(&facets_json)));
                    }

                    if let Some(sort) = options.sort {
                        let index = match sort {
                            ModrinthSortOrder::Relevance => "relevance",
                            ModrinthSortOrder::Downloads => "downloads",
                            ModrinthSortOrder::Follows => "follows",
                            ModrinthSortOrder::Newest => "newest",
                            ModrinthSortOrder::Updated => "updated",
                        };
                        url.push_str(&format!("&index={}", index));
                    }

                    if let Some(offset) = options.offset {
                        url.push_str(&format!("&offset={}", offset));
                    }

                    if let Some(limit) = options.limit {
                        url.push_str(&format!("&limit={}", limit));
                    }

                    let response_text = retry_async(
                        || async {
                            let res = client.get(&url).send().await?;
                            let text = res.text().await.map_err(|e| anyhow!(e))?;
                            Ok(text)
                        },
                        3,
                        Duration::from_secs(2),
                        &format!("Modrinth search: {}", options.query),
                    )
                    .await?;

                    let response: serde_json::Value = serde_json::from_str(&response_text)?;
                    let hits = response["hits"]
                        .as_array()
                        .ok_or_else(|| anyhow!("Missing hits field"))?;

                    let projects: Vec<ModrinthProject> = hits
                        .iter()
                        .map(|h| {
                            let p_type = match h["project_type"].as_str().unwrap_or_default() {
                                "mod" => ModrinthProjectType::Mod,
                                "plugin" => ModrinthProjectType::Plugin,
                                "resourcepack" => ModrinthProjectType::ResourcePack,
                                "datapack" => ModrinthProjectType::DataPack,
                                _ => ModrinthProjectType::Mod, // Default
                            };

                            ModrinthProject {
                                id: h["project_id"].as_str().unwrap_or_default().to_string(),
                                slug: h["slug"].as_str().unwrap_or_default().to_string(),
                                title: h["title"].as_str().unwrap_or_default().to_string(),
                                description: h["description"]
                                    .as_str()
                                    .unwrap_or_default()
                                    .to_string(),
                                downloads: h["downloads"].as_u64().unwrap_or(0),
                                icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
                                screenshot_urls: h["gallery"].as_array().map(|gallery| {
                                    gallery
                                        .iter()
                                        .filter_map(|item| {
                                            item["url"].as_str().map(|s| s.to_string())
                                        })
                                        .collect()
                                }),
                                author: h["author"].as_str().unwrap_or_default().to_string(),
                                project_type: p_type,
                                categories: h["categories"].as_array().map(|cats| {
                                    cats.iter()
                                        .filter_map(|c| c.as_str().map(|s| s.to_string()))
                                        .collect()
                                }),
                            }
                        })
                        .collect();

                    Ok(projects)
                }
            })
            .await
    }

    pub async fn get_project(&self, id: &str) -> Result<ModrinthProject> {
        let cache_key = format!("modrinth_common_project_{}", id);
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let id = id.to_string();

        self.cache
            .fetch_with_cache(cache_key, Duration::from_secs(3600), move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let id = id.clone();
                async move {
                    let url = format!("{}/project/{}", base_url, id);
                    let response: serde_json::Value = retry_async(
                        || async {
                            client
                                .get(&url)
                                .send()
                                .await?
                                .json()
                                .await
                                .map_err(|e| anyhow!(e))
                        },
                        3,
                        Duration::from_secs(2),
                        &format!("Get Modrinth project: {}", id),
                    )
                    .await?;

                    let p_type = match response["project_type"].as_str().unwrap_or_default() {
                        "mod" => ModrinthProjectType::Mod,
                        "plugin" => ModrinthProjectType::Plugin,
                        "resourcepack" => ModrinthProjectType::ResourcePack,
                        "datapack" => ModrinthProjectType::DataPack,
                        _ => ModrinthProjectType::Mod,
                    };

                    Ok(ModrinthProject {
                        id: response["id"].as_str().unwrap_or_default().to_string(),
                        slug: response["slug"].as_str().unwrap_or_default().to_string(),
                        title: response["title"].as_str().unwrap_or_default().to_string(),
                        description: response["description"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        downloads: response["downloads"].as_u64().unwrap_or(0),
                        icon_url: response["icon_url"].as_str().map(|s| s.to_string()),
                        screenshot_urls: response["gallery"].as_array().map(|gallery| {
                            gallery
                                .iter()
                                .filter_map(|item| item["url"].as_str().map(|s| s.to_string()))
                                .collect()
                        }),
                        author: String::new(), // Author is not directly in project response in the same way as search
                        project_type: p_type,
                        categories: response["categories"].as_array().map(|cats| {
                            cats.iter()
                                .filter_map(|c| c.as_str().map(|s| s.to_string()))
                                .collect()
                        }),
                    })
                }
            })
            .await
    }

    pub async fn get_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ModrinthVersion>> {
        let cache_key = format!(
            "modrinth_common_versions_{}_v:{:?}_lo:{:?}",
            project_id, game_version, loader
        );
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let project_id = project_id.to_string();
        let game_version = game_version.map(|s| s.to_string());
        let loader = loader.map(|s| s.to_string());

        self.cache
            .fetch_with_cache(cache_key, Duration::from_secs(3600), move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let project_id = project_id.clone();
                let game_version = game_version.clone();
                let loader = loader.clone();
                async move {
                    let mut url = format!("{}/project/{}/version", base_url, project_id);
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

                    let versions: Vec<ModrinthVersion> = retry_async(
                        || async {
                            client
                                .get(&url)
                                .send()
                                .await?
                                .json()
                                .await
                                .map_err(|e| anyhow!(e))
                        },
                        3,
                        Duration::from_secs(2),
                        &format!("Get Modrinth versions: {}", project_id),
                    )
                    .await?;

                    Ok(versions)
                }
            })
            .await
    }

    pub async fn get_dependencies(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<(ModrinthProject, String)>> {
        let cache_key = format!(
            "modrinth_common_deps_{}_v:{:?}_lo:{:?}",
            project_id, game_version, loader
        );

        if let Ok(Some(cached)) = self
            .cache
            .get::<Vec<(ModrinthProject, String)>>(&cache_key)
            .await
        {
            return Ok(cached);
        }

        let mut resolved = Vec::new();

        // If version/loader provided, try to get specific version dependencies
        if let (Some(gv), Some(l)) = (game_version, loader) {
            let versions = self.get_versions(project_id, Some(gv), Some(l)).await?;
            let l_lower = l.to_lowercase();

            if let Some(version) = versions.into_iter().find(|v| {
                v.game_versions.contains(&gv.to_string())
                    && v.loaders.iter().any(|vl| vl.to_lowercase() == l_lower)
            }) {
                for dep in version.dependencies {
                    if let Some(dep_id) = dep.project_id {
                        if dep.dependency_type == "required" || dep.dependency_type == "optional" {
                            if let Ok(project) = self.get_project(&dep_id).await {
                                resolved.push((project, dep.dependency_type));
                            }
                        }
                    }
                }
                let _ = self.cache.set(cache_key, resolved.clone()).await;
                return Ok(resolved);
            }
        }

        // Fallback to project-wide dependencies
        let url = format!("{}/project/{}/dependencies", self.base_url, project_id);
        let response: serde_json::Value = retry_async(
            || async {
                self.client
                    .get(&url)
                    .send()
                    .await?
                    .json()
                    .await
                    .map_err(|e| anyhow!(e))
            },
            3,
            Duration::from_secs(2),
            &format!("Get Modrinth dependencies: {}", project_id),
        )
        .await?;

        if let (Some(projects), Some(versions)) = (
            response["projects"].as_array(),
            response["versions"].as_array(),
        ) {
            for p_json in projects {
                let id = p_json["id"].as_str().unwrap_or_default().to_string();
                let dep_type = versions
                    .iter()
                    .find(|v| v["project_id"].as_str() == Some(&id))
                    .and_then(|v| v["dependency_type"].as_str())
                    .unwrap_or("required")
                    .to_string();

                let p_type = match p_json["project_type"].as_str().unwrap_or_default() {
                    "mod" => ModrinthProjectType::Mod,
                    "plugin" => ModrinthProjectType::Plugin,
                    "resourcepack" => ModrinthProjectType::ResourcePack,
                    "datapack" => ModrinthProjectType::DataPack,
                    _ => ModrinthProjectType::Mod,
                };

                resolved.push((
                    ModrinthProject {
                        id,
                        slug: p_json["slug"].as_str().unwrap_or_default().to_string(),
                        title: p_json["title"].as_str().unwrap_or_default().to_string(),
                        description: p_json["description"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        downloads: p_json["downloads"].as_u64().unwrap_or(0),
                        icon_url: p_json["icon_url"].as_str().map(|s| s.to_string()),
                        screenshot_urls: None,
                        author: String::new(),
                        project_type: p_type,
                        categories: p_json["categories"].as_array().map(|cats| {
                            cats.iter()
                                .filter_map(|c| c.as_str().map(|s| s.to_string()))
                                .collect()
                        }),
                    },
                    dep_type,
                ));
            }
        }

        let _ = self.cache.set(cache_key, resolved.clone()).await;
        Ok(resolved)
    }
}
