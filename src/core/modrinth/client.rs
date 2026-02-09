use super::types::*;
use crate::cache::CacheManager;
use anyhow::{Context, Result, anyhow};
use ferinth::structures::{project::Project, search::Response as SearchResponse, version::Version};
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use std::sync::Arc;
use std::time::Duration;

pub struct ModrinthClient {
    base_url: String,
    pub cache: Arc<CacheManager>,
}

impl ModrinthClient {
    pub fn new(cache: Arc<CacheManager>) -> Self {
        Self::with_base_url("https://api.modrinth.com/v2".to_string(), cache)
    }

    pub fn with_base_url(base_url: String, cache: Arc<CacheManager>) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            cache,
        }
    }

    pub async fn search(&self, options: &ModrinthSearchOptions) -> Result<Vec<ModrinthProject>> {
        let cache_key = format!("modrinth_search_{}", options.cache_key());
        let options = options.clone();
        let base_url = self.base_url.clone();
        let client = self.cache.get_client().clone();

        self.cache
            .fetch_with_options(cache_key, Duration::from_secs(3600), false, move || {
                let client = client.clone();
                let options = options.clone();
                let base_url = base_url.clone();
                async move {
                    let mut query_params = vec![
                        ("query", options.query.clone()),
                        ("offset", options.offset.unwrap_or(0).to_string()),
                        ("limit", options.limit.unwrap_or(20).to_string()),
                    ];

                    if let Some(sort) = &options.sort {
                        let sort_str = match sort {
                            ModrinthSortOrder::Relevance => "relevance",
                            ModrinthSortOrder::Downloads => "downloads",
                            ModrinthSortOrder::Follows => "follows",
                            ModrinthSortOrder::Newest => "newest",
                            ModrinthSortOrder::Updated => "updated",
                        };
                        query_params.push(("index", sort_str.to_string()));
                    }

                    let mut facet_groups = Vec::new();

                    if let Some(project_type) = &options.project_type {
                        let type_str = match project_type {
                            ModrinthProjectType::Mod => "mod",
                            ModrinthProjectType::Plugin => "plugin",
                            ModrinthProjectType::ResourcePack => "resourcepack",
                            ModrinthProjectType::DataPack => "datapack",
                            ModrinthProjectType::Modpack => "modpack",
                            ModrinthProjectType::Shader => "shader",
                        };
                        facet_groups.push(vec![format!("project_type:{}", type_str)]);
                    }

                    if let Some(version) = &options.game_version {
                        facet_groups.push(vec![format!("versions:{}", version)]);
                    }

                    if let Some(loader) = &options.loader {
                        facet_groups.push(vec![format!("categories:{}", loader.to_lowercase())]);
                    }

                    if let Some(facets) = &options.facets {
                        if !facets.is_empty() {
                            // If we already have facets from the UI, we should add them as well.
                            // The UI facets are usually category filters.
                            for f in facets {
                                facet_groups.push(vec![f.clone()]);
                            }
                        }
                    }

                    if !facet_groups.is_empty() {
                        let facets_str = format!(
                            "[{}]",
                            facet_groups
                                .iter()
                                .map(|group| {
                                    format!(
                                        "[{}]",
                                        group
                                            .iter()
                                            .map(|f| format!("\"{}\"", f))
                                            .collect::<Vec<_>>()
                                            .join(",")
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(",")
                        );
                        query_params.push(("facets", facets_str));
                    }

                    let url = format!("{}/search", base_url);
                    let response = client
                        .get(&url)
                        .query(&query_params)
                        .send()
                        .await
                        .context("Failed to send search request")?;

                    if !response.status().is_success() {
                        return Err(anyhow!(
                            "Search request failed with status: {}",
                            response.status()
                        ));
                    }

                    let search_response: SearchResponse = response
                        .json()
                        .await
                        .context("Failed to parse search response")?;

                    Ok(search_response.hits.into_iter().map(Into::into).collect())
                }
            })
            .await
    }

    pub async fn get_project(&self, id: &str) -> Result<ModrinthProject> {
        let cache_key = format!("modrinth_project_{}", id);
        let client = self.cache.get_client().clone();
        let url = format!("{}/project/{}", self.base_url, id);

        self.cache
            .fetch_with_cache(cache_key, Duration::from_secs(3600), move || {
                let client = client.clone();
                let url = url.clone();
                async move {
                    let response = client
                        .get(&url)
                        .send()
                        .await
                        .context("Failed to send project request")?;

                    if !response.status().is_success() {
                        return Err(anyhow!(
                            "Project request failed with status: {}",
                            response.status()
                        ));
                    }

                    let p: Project = response
                        .json()
                        .await
                        .context("Failed to parse project response")?;
                    Ok(p.into())
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
            "modrinth_versions_{}_v:{:?}_lo:{:?}",
            project_id, game_version, loader
        );
        let client = self.cache.get_client().clone();
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
                    let mut query_params = Vec::new();
                    if let Some(gv) = &game_version {
                        query_params.push(("game_versions", format!("[\"{}\"]", gv)));
                    }
                    if let Some(l) = &loader {
                        query_params.push(("loaders", format!("[\"{}\"]", l)));
                    }

                    let url = format!("{}/project/{}/version", base_url, project_id);
                    let response = client
                        .get(&url)
                        .query(&query_params)
                        .send()
                        .await
                        .context("Failed to send versions request")?;

                    if !response.status().is_success() {
                        return Err(anyhow!(
                            "Versions request failed with status: {}",
                            response.status()
                        ));
                    }

                    let versions: Vec<Version> = response
                        .json()
                        .await
                        .context("Failed to parse versions response")?;

                    Ok(versions.into_iter().map(Into::into).collect())
                }
            })
            .await
    }

    pub fn get_dependencies<'a>(
        &'a self,
        project_id: &'a str,
        game_version: Option<&'a str>,
        loader: Option<&'a str>,
        _project_type: Option<ModrinthProjectType>,
    ) -> BoxFuture<'a, Result<Vec<(ModrinthProject, String)>>> {
        async move {
            // 1. Get versions to find a suitable one
            let versions = self.get_versions(project_id, game_version, loader).await?;
            let version = versions
                .first()
                .ok_or_else(|| anyhow!("No versions found for project {}", project_id))?;

            // 2. Get dependencies for that version
            let url = format!("{}/version/{}", self.base_url, version.id);
            let response = self
                .cache
                .get_client()
                .get(&url)
                .send()
                .await
                .context("Failed to send version request for dependencies")?;

            if !response.status().is_success() {
                return Err(anyhow!(
                    "Version request failed with status: {}",
                    response.status()
                ));
            }

            let version_data: Version = response
                .json()
                .await
                .context("Failed to parse version response for dependencies")?;

            // 3. Resolve each dependency to a ModrinthProject
            let mut resolved = Vec::new();
            for dep in version_data.dependencies {
                if let Some(dep_proj_id) = dep.project_id {
                    let project = self.get_project(&dep_proj_id).await?;
                    resolved.push((project, format!("{:?}", dep.dependency_type)));
                }
            }

            Ok(resolved)
        }
        .boxed()
    }
}
