use super::HangarClient;
use crate::plugins::types::{
    PluginProvider, Project, ResolvedDependency, SearchOptions, SortOrder,
};
use anyhow::{Result, anyhow};
use std::sync::Arc;

impl HangarClient {
    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let cache_key = format!("hangar_search_{}", options.cache_key());
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let query = options.query.clone();
        let offset = options.offset;
        let limit = options.limit;
        let sort = options.sort;

        self.cache
            .fetch_with_options(
                cache_key,
                std::time::Duration::from_secs(3600),
                false,
                move || {
                    let client = client.clone();
                    let base_url = base_url.clone();
                    let query = query.clone();
                    async move {
                        let mut url =
                            format!("{}/projects?q={}", base_url, urlencoding::encode(&query));

                        if let Some(offset) = offset {
                            url.push_str(&format!("&offset={}", offset));
                        }

                        if let Some(limit) = limit {
                            url.push_str(&format!("&limit={}", limit));
                        }

                        // Hangar doesn't have a direct "sort" parameter that matches ours exactly,
                        // but it has 'sort' parameter with values like 'stars', 'downloads', 'newest', 'updated', 'recent_views', 'recent_downloads'
                        if let Some(sort) = sort {
                            let sort_val = match sort {
                                SortOrder::Relevance => "recent_downloads", // Default to recent downloads for relevance
                                SortOrder::Downloads => "downloads",
                                SortOrder::Follows => "stars",
                                SortOrder::Newest => "newest",
                                SortOrder::Updated => "updated",
                            };
                            url.push_str(&format!("&sort={}", sort_val));
                        }

                        let response: serde_json::Value =
                            client.get(&url).send().await?.json().await?;

                        let result = response["result"].as_array().ok_or_else(|| {
                            anyhow!("Invalid response from Hangar: missing 'result' field")
                        })?;

                        let projects: Vec<Project> = result
                            .iter()
                            .map(|h| {
                                let owner = h["namespace"]["owner"].as_str().unwrap_or_default();
                                let slug = h["namespace"]["slug"].as_str().unwrap_or_default();
                                Project {
                                    // Use owner/slug as ID for Hangar to make API calls easier later
                                    id: format!("{}/{}", owner, slug),
                                    slug: slug.to_string(),
                                    title: h["name"].as_str().unwrap_or_default().to_string(),
                                    description: h["description"]
                                        .as_str()
                                        .unwrap_or_default()
                                        .to_string(),
                                    downloads: h["stats"]["downloads"].as_u64().unwrap_or(0),
                                    icon_url: h["avatarUrl"].as_str().map(|s| s.to_string()),
                                    screenshot_urls: None,
                                    author: owner.to_string(),
                                    provider: PluginProvider::Hangar,
                                    categories: None,
                                }
                            })
                            .collect();

                        Ok(projects)
                    }
                },
            )
            .await
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let cache_key = format!("hangar_project_{}", id);
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let id = id.to_string();

        self.cache
            .fetch_with_cache(cache_key, std::time::Duration::from_secs(3600), move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let id = id.clone();
                async move {
                    let url = format!("{}/projects/{}", base_url, id);
                    let h: serde_json::Value = client.get(&url).send().await?.json().await?;

                    let owner = h["namespace"]["owner"].as_str().unwrap_or_default();
                    let slug = h["namespace"]["slug"].as_str().unwrap_or_default();

                    let project = Project {
                        id: id.clone(),
                        slug: slug.to_string(),
                        title: h["name"].as_str().unwrap_or_default().to_string(),
                        description: h["description"].as_str().unwrap_or_default().to_string(),
                        downloads: h["stats"]["downloads"].as_u64().unwrap_or(0),
                        icon_url: h["avatarUrl"].as_str().map(|s| s.to_string()),
                        screenshot_urls: None,
                        author: owner.to_string(),
                        provider: PluginProvider::Hangar,
                        categories: None,
                    };

                    Ok(project)
                }
            })
            .await
    }

    pub async fn get_dependencies(
        &self,
        project_id: &str,
        loader: Option<&str>,
    ) -> Result<Vec<ResolvedDependency>> {
        let cache_key = format!("hangar_dependencies_{}_lo:{:?}", project_id, loader);
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let project_id = project_id.to_string();
        let loader = loader.map(|s| s.to_uppercase());
        let cache = Arc::clone(&self.cache);

        self.cache
            .fetch_with_cache(cache_key, std::time::Duration::from_secs(3600), move || {
                let client = client.clone();
                let base_url = base_url.clone();
                let project_id = project_id.clone();
                let loader = loader.clone();
                let cache = Arc::clone(&cache);
                async move {
                    // Hangar dependencies are listed per version. We'll get the latest version and its dependencies.
                    let url = format!("{}/projects/{}/versions?limit=1", base_url, project_id);
                    let response: serde_json::Value = client.get(&url).send().await?.json().await?;

                    let result = response["result"]
                        .as_array()
                        .ok_or_else(|| anyhow!("Invalid response from Hangar"))?;
                    if result.is_empty() {
                        return Ok(vec![]);
                    }

                    let latest_version = &result[0];
                    let mut resolved_deps = Vec::new();

                    // Map our loader to Hangar platforms
                    // Hangar platforms: PAPER, WATERFALL, VELOCITY
                    let platform = match loader.as_deref() {
                        Some("PAPER") | Some("SPIGOT") | Some("BUKKIT") | Some("PURPUR") => "PAPER",
                        Some("VELOCITY") => "VELOCITY",
                        Some("WATERFALL") | Some("BUNGEECORD") => "WATERFALL",
                        _ => "PAPER", // Default to PAPER
                    };

                    // Check for plugin dependencies in the matched platform
                    if let Some(plugin_deps) = latest_version["pluginDependencies"].get(platform) {
                        if let Some(deps_array) = plugin_deps.as_array() {
                            for dep in deps_array {
                                let name = dep["name"].as_str().unwrap_or("Unknown").to_string();
                                let required = dep["required"].as_bool().unwrap_or(true);
                                let dependency_type =
                                    if required { "required" } else { "optional" }.to_string();

                                if let Some(namespace) = dep.get("namespace") {
                                    let owner = namespace["owner"].as_str().unwrap_or_default();
                                    let slug = namespace["slug"].as_str().unwrap_or_default();

                                    if !owner.is_empty() && !slug.is_empty() {
                                        let dep_project_id = format!("{}/{}", owner, slug);

                                        // We'll create a temporary client to call get_project
                                        // This is a bit inefficient but keeps things clean
                                        let temp_client = super::HangarClient::with_base_url(
                                            base_url.clone(),
                                            Arc::clone(&cache),
                                        );

                                        if let Ok(project) =
                                            temp_client.get_project(&dep_project_id).await
                                        {
                                            resolved_deps.push(ResolvedDependency {
                                                project,
                                                dependency_type,
                                            });
                                        } else {
                                            // Fallback to partial project if fetch fails
                                            resolved_deps.push(ResolvedDependency {
                                                project: Project {
                                                    id: dep_project_id,
                                                    slug: slug.to_string(),
                                                    title: name,
                                                    description: String::new(),
                                                    downloads: 0,
                                                    icon_url: None,
                                                    screenshot_urls: None,
                                                    author: owner.to_string(),
                                                    provider: PluginProvider::Hangar,
                                                    categories: None,
                                                },
                                                dependency_type,
                                            });
                                        }
                                    }
                                } else if let Some(external_url) = dep["externalUrl"].as_str() {
                                    // External dependency
                                    resolved_deps.push(ResolvedDependency {
                                        project: Project {
                                            id: external_url.to_string(),
                                            slug: name.to_lowercase().replace(' ', "-"),
                                            title: name,
                                            description: format!(
                                                "External dependency: {}",
                                                external_url
                                            ),
                                            downloads: 0,
                                            icon_url: None,
                                            screenshot_urls: None,
                                            author: "External".to_string(),
                                            provider: PluginProvider::Hangar,
                                            categories: None,
                                        },
                                        dependency_type,
                                    });
                                }
                            }
                        }
                    }

                    Ok(resolved_deps)
                }
            })
            .await
    }
}
