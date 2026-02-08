use super::SpigetClient;
use crate::plugins::types::{PluginProvider, Project, ResolvedDependency, SearchOptions};
use anyhow::Result;
use std::sync::Arc;

impl SpigetClient {
    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let cache_key = format!("spiget_search_{}", options.cache_key());
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let query = options.query.clone();
        let facets = options.facets.clone();
        let limit = options.limit;
        let offset = options.offset;

        self.cache
            .fetch_with_options(
                cache_key,
                std::time::Duration::from_secs(3600),
                false,
                move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let query = query.clone();
                let facets = facets.clone();
                async move {
                    let size = limit.unwrap_or(20);
                    let page = (offset.unwrap_or(0) / size) + 1;

                    let url = if query.trim().is_empty() {
                        if let Some(facets) = &facets {
                            let category = facets
                                .iter()
                                .find(|f| f.starts_with("categories:"))
                                .and_then(|f| f.strip_prefix("categories:"));

                            if let Some(cat) = category {
                                let cat_id = match cat {
                                    "chat" | "social" => "11",
                                    "economy" => "12",
                                    "gameplay" | "adventure" | "game-mechanics" | "magic"
                                    | "minigame" => "13",
                                    "management" => "14",
                                    "protection" => "15",
                                    "utility" | "optimization" => "16",
                                    "world-management" | "worldgen" => "17",
                                    "library" => "19",
                                    "admin" => "10",
                                    "misc" => "18",
                                    _ => cat,
                                };
                                format!(
                                    "{}/categories/{}/resources?size={}&page={}&sort=-downloads",
                                    base_url, cat_id, size, page
                                )
                            } else {
                                format!(
                                    "{}/resources?size={}&page={}&sort=-downloads",
                                    base_url, size, page
                                )
                            }
                        } else {
                            format!(
                                "{}/resources?size={}&page={}&sort=-downloads",
                                base_url, size, page
                            )
                        }
                    } else {
                        format!(
                            "{}/search/resources/{}?field=name&size={}&page={}",
                            base_url,
                            urlencoding::encode(&query),
                            size,
                            page
                        )
                    };

                    let response = client.get(&url).send().await?;

                    if response.status() == reqwest::StatusCode::NOT_FOUND {
                        return Ok(vec![]);
                    }

                    let response = response.error_for_status()?;
                    let json = response.json::<Vec<serde_json::Value>>().await?;

                    let projects: Vec<Project> = json
                        .into_iter()
                        .map(|h| Project {
                            id: h["id"].as_u64().unwrap_or(0).to_string(),
                            slug: h["name"]
                                .as_str()
                                .unwrap_or_default()
                                .to_lowercase()
                                .replace(' ', "-"),
                            title: h["name"].as_str().unwrap_or_default().to_string(),
                            description: h["tag"].as_str().unwrap_or_default().to_string(),
                            downloads: h["downloads"].as_u64().unwrap_or(0),
                            icon_url: h["icon"]["url"].as_str().filter(|s| !s.is_empty()).map(
                                |s| {
                                    if s.starts_with("http") {
                                        s.to_string()
                                    } else {
                                        format!("https://www.spigotmc.org/{}", s)
                                    }
                                },
                            ),
                            screenshot_urls: None,
                            author: format!("User {}", h["author"]["id"]),
                            provider: PluginProvider::Spiget,
                            categories: h["category"]["id"].as_u64().map(|id| vec![id.to_string()]),
                        })
                        .collect();

                    Ok(projects)
                }
            })
            .await
    }

    pub async fn get_dependencies(&self, resource_id: &str) -> Result<Vec<ResolvedDependency>> {
        let cache_key = format!("spiget_dependencies_{}", resource_id);
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let resource_id = resource_id.to_string();
        let cache = Arc::clone(&self.cache);

        self.cache
            .fetch_with_cache(cache_key, std::time::Duration::from_secs(3600), move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let resource_id = resource_id.clone();
                let cache = Arc::clone(&cache);
                async move {
                    let url = format!("{}/resources/{}/dependencies", base_url, resource_id);
                    let response = client.get(&url).send().await?;

                    if response.status() == reqwest::StatusCode::NOT_FOUND {
                        return Ok(vec![]);
                    }

                    let response = response.error_for_status()?;
                    let json = response.json::<Vec<serde_json::Value>>().await?;

                    let mut resolved_deps = Vec::new();
                    for dep in json {
                        if let Some(id) = dep["id"].as_u64() {
                            // We'll use a temporary client to avoid recursive fetch issues if any,
                            // though fetch_with_cache handles concurrency.
                            let temp_client = super::SpigetClient::with_base_url(
                                base_url.clone(),
                                Arc::clone(&cache),
                            );
                            if let Ok(project) = temp_client.get_project(&id.to_string()).await {
                                resolved_deps.push(ResolvedDependency {
                                    project,
                                    dependency_type: "required".to_string(),
                                });
                            }
                        }
                    }

                    Ok(resolved_deps)
                }
            })
            .await
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let cache_key = format!("spiget_project_{}", id);
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let id = id.to_string();

        self.cache
            .fetch_with_cache(cache_key, std::time::Duration::from_secs(3600), move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let id = id.clone();
                async move {
                    let url = format!("{}/resources/{}", base_url, id);
                    let response = client.get(&url).send().await?.error_for_status()?;
                    let h = response.json::<serde_json::Value>().await?;

                    let project = Project {
                        id: h["id"].as_u64().unwrap_or(0).to_string(),
                        slug: h["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_lowercase()
                            .replace(' ', "-"),
                        title: h["name"].as_str().unwrap_or_default().to_string(),
                        description: h["tag"].as_str().unwrap_or_default().to_string(),
                        downloads: h["downloads"].as_u64().unwrap_or(0),
                        icon_url: h["icon"]["url"]
                            .as_str()
                            .filter(|s| !s.is_empty())
                            .map(|s| {
                                if s.starts_with("http") {
                                    s.to_string()
                                } else {
                                    format!("https://www.spigotmc.org/{}", s)
                                }
                            }),
                        screenshot_urls: None,
                        author: format!("User {}", h["author"]["id"]),
                        provider: PluginProvider::Spiget,
                        categories: h["category"]["id"].as_u64().map(|id| vec![id.to_string()]),
                    };

                    Ok(project)
                }
            })
            .await
    }

    pub async fn get_latest_version(&self, resource_id: &str) -> Result<(String, String)> {
        let url = format!(
            "{}/resources/{}/versions/latest",
            self.base_url, resource_id
        );
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let json = response.json::<serde_json::Value>().await?;

        let id = json["id"].as_u64().unwrap_or(0).to_string();
        let name = json["name"].as_str().unwrap_or_default().to_string();

        Ok((id, name))
    }
}
