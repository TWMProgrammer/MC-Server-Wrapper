use super::CurseForgeClient;
use crate::mods::types::{ModProvider, Project, SearchOptions, SortOrder};
use anyhow::{Result, anyhow};

impl CurseForgeClient {
    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let cache_key = format!("curseforge_search_{}", options.cache_key());
        if let Ok(Some(cached)) = self.cache.get::<Vec<Project>>(&cache_key).await {
            return Ok(cached);
        }

        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("CurseForge API key not provided"))?;

        let mut query_params = vec![
            ("gameId", "432".to_string()), // Minecraft
            ("searchFilter", options.query.clone()),
            ("classId", "6".to_string()), // Mods
        ];

        if let Some(sort) = options.sort {
            let sort_field = match sort {
                SortOrder::Relevance => "0", // Featured
                SortOrder::Downloads => "2", // Popularity
                SortOrder::Follows => "2",   // Popularity (closest match)
                SortOrder::Newest => "1",    // Newest
                SortOrder::Updated => "3",   // Updated
            };
            query_params.push(("sortField", sort_field.to_string()));
            query_params.push(("sortOrder", "desc".to_string()));
        }

        if let Some(offset) = options.offset {
            query_params.push(("index", offset.to_string()));
        }

        if let Some(limit) = options.limit {
            query_params.push(("pageSize", limit.to_string()));
        }

        if let Some(version) = &options.game_version {
            query_params.push(("gameVersion", version.clone()));
        }

        if let Some(loader) = &options.loader {
            let loader_type = match loader.to_lowercase().as_str() {
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

        if let Some(facets) = &options.facets {
            for facet in facets {
                if facet.starts_with("categories:") {
                    let cat = facet.strip_prefix("categories:").unwrap_or("");
                    let cat_id = match cat {
                        "adventure" => Some("421"),
                        "decoration" => Some("424"),
                        "equipment" => Some("423"),
                        "food" => Some("427"),
                        "library" => Some("422"),
                        "magic" => Some("419"),
                        "management" => Some("426"),
                        "optimization" => Some("417"),
                        "storage" => Some("425"),
                        "technology" => Some("418"),
                        "utility" => Some("416"),
                        "worldgen" => Some("409"),
                        _ => None,
                    };
                    if let Some(id) = cat_id {
                        query_params.push(("categoryId", id.to_string()));
                    }
                }
            }
        }

        let url = "https://api.curseforge.com/v1/mods/search";
        let response = self
            .client
            .get(url)
            .header("x-api-key", api_key)
            .query(&query_params)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let data = response["data"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid response from CurseForge"))?;

        let projects: Vec<Project> = data
            .iter()
            .map(|h| Project {
                id: h["id"].as_u64().unwrap_or(0).to_string(),
                slug: h["slug"].as_str().unwrap_or_default().to_string(),
                title: h["name"].as_str().unwrap_or_default().to_string(),
                description: h["summary"].as_str().unwrap_or_default().to_string(),
                downloads: h["downloadCount"].as_u64().unwrap_or(0),
                icon_url: h["logo"]["url"].as_str().map(|s| s.to_string()),
                screenshot_urls: h["screenshots"].as_array().map(|screenshots| {
                    screenshots
                        .iter()
                        .filter_map(|item| item["url"].as_str().map(|s| s.to_string()))
                        .collect()
                }),
                author: h["authors"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|a| a["name"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                provider: ModProvider::CurseForge,
                categories: h["categories"].as_array().map(|cats| {
                    cats.iter()
                        .filter_map(|c| c["name"].as_str().map(|s| s.to_string()))
                        .collect()
                }),
            })
            .collect();

        let _ = self.cache.set(cache_key, projects.clone()).await;
        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let cache_key = format!("curseforge_project_{}", id);
        if let Ok(Some(cached)) = self.cache.get::<Project>(&cache_key).await {
            return Ok(cached);
        }

        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("CurseForge API key not provided"))?;
        let url = format!("https://api.curseforge.com/v1/mods/{}", id);
        let response = self
            .client
            .get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let h = &response["data"];
        let project = Project {
            id: h["id"].as_u64().unwrap_or(0).to_string(),
            slug: h["slug"].as_str().unwrap_or_default().to_string(),
            title: h["name"].as_str().unwrap_or_default().to_string(),
            description: h["summary"].as_str().unwrap_or_default().to_string(),
            downloads: h["downloadCount"].as_u64().unwrap_or(0),
            icon_url: h["logo"]["url"].as_str().map(|s| s.to_string()),
            screenshot_urls: h["screenshots"].as_array().map(|screenshots| {
                screenshots
                    .iter()
                    .filter_map(|item| item["url"].as_str().map(|s| s.to_string()))
                    .collect()
            }),
            author: h["authors"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|a| a["name"].as_str())
                .unwrap_or("Unknown")
                .to_string(),
            provider: ModProvider::CurseForge,
            categories: h["categories"].as_array().map(|cats| {
                cats.iter()
                    .filter_map(|c| c["name"].as_str().map(|s| s.to_string()))
                    .collect()
            }),
        };

        let _ = self.cache.set(cache_key, project.clone()).await;
        Ok(project)
    }
}
