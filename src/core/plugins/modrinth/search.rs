use anyhow::{Result, anyhow};
use super::ModrinthClient;
use crate::plugins::types::{Project, PluginProvider, ResolvedDependency, SearchOptions, SortOrder};

impl ModrinthClient {
    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let cache_key = format!("modrinth_search_{}", options.cache_key());
        if let Ok(Some(cached)) = self.cache.get::<Vec<Project>>(&cache_key).await {
            return Ok(cached);
        }

        let mut url = format!("https://api.modrinth.com/v2/search?query={}", urlencoding::encode(&options.query));

        let mut and_groups: Vec<Vec<String>> = Vec::new();

        if let Some(facets) = &options.facets {
            if !facets.is_empty() {
                for f in facets {
                    and_groups.push(vec![f.clone()]);
                }
            }
        }

        // Add project_type filter if not present
        let mut has_type = false;
        for group in &and_groups {
            if group.iter().any(|f| f.starts_with("project_type:")) {
                has_type = true;
                break;
            }
        }
        
        if !has_type {
            // Strictly restrict to plugin type as requested
            and_groups.push(vec!["project_type:plugin".to_string()]);
        }

        if let Some(version) = &options.game_version {
            if !version.is_empty() {
                and_groups.push(vec![format!("versions:{}", version)]);
            }
        }

        if let Some(loader) = &options.loader {
            if !loader.is_empty() {
                // For plugins, loaders might be 'paper', 'purpur', 'spigot', 'velocity', 'bungeecord'
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

        let response_text = self.client.get(&url).send().await?.text().await?;
        let response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Modrinth response: {}. Body: {}", e, response_text))?;
        
        let hits = response["hits"].as_array().ok_or_else(|| anyhow!("Invalid response from Modrinth: missing 'hits' field"))?;
        
        let projects: Vec<Project> = hits.iter().map(|h| Project {
            id: h["project_id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
            author: h["author"].as_str().unwrap_or_default().to_string(),
            provider: PluginProvider::Modrinth,
        }).collect();

        let _ = self.cache.set(cache_key, projects.clone()).await;
        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let cache_key = format!("modrinth_project_{}", id);
        if let Ok(Some(cached)) = self.cache.get::<Project>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("https://api.modrinth.com/v2/project/{}", id);
        let h = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        let project = Project {
            id: h["id"].as_str().unwrap_or_default().to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["title"].as_str().unwrap_or_default().to_string(),
            description: h["description"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloads"].as_u64().unwrap_or(0),
            icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
            author: String::new(), // Author is in a separate field in project API
            provider: PluginProvider::Modrinth,
        };

        let _ = self.cache.set(cache_key, project.clone()).await;
        Ok(project)
    }

    pub async fn get_dependencies(&self, project_id: &str) -> Result<Vec<ResolvedDependency>> {
        let cache_key = format!("modrinth_dependencies_{}", project_id);
        if let Ok(Some(cached)) = self.cache.get::<Vec<ResolvedDependency>>(&cache_key).await {
            return Ok(cached);
        }

        let url = format!("https://api.modrinth.com/v2/project/{}/dependencies", project_id);
        let response = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;
        
        let projects_json = response["projects"].as_array().ok_or_else(|| anyhow!("Invalid dependencies response"))?;
        let versions_json = response["versions"].as_array().ok_or_else(|| anyhow!("Invalid dependencies response"))?;
        
        let mut resolved_deps = Vec::new();
        
        for h in projects_json {
            // Check if this project is actually a plugin
            let project_type = h["project_type"].as_str().unwrap_or_default();
            if project_type != "plugin" {
                continue;
            }

            let id = h["id"].as_str().unwrap_or_default().to_string();
            
            // Find the dependency type from the versions array
            // The versions array contains objects with 'project_id' and 'dependency_type'
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
                    icon_url: h["icon_url"].as_str().map(|s| s.to_string()),
                    author: String::new(),
                    provider: PluginProvider::Modrinth,
                },
                dependency_type,
            });
        }

        let _ = self.cache.set(cache_key, resolved_deps.clone()).await;
        Ok(resolved_deps)
    }
}
