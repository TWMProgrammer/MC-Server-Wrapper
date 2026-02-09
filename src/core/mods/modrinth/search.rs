use super::ModrinthClient;
use crate::modrinth::{ModrinthProjectType, ModrinthSearchOptions, ModrinthSortOrder};
use crate::mods::types::{ModProvider, Project, SearchOptions, SortOrder};
use anyhow::Result;

impl ModrinthClient {
    pub async fn search(&self, options: &SearchOptions) -> Result<Vec<Project>> {
        let common_options = ModrinthSearchOptions {
            query: options.query.clone(),
            facets: options.facets.clone(),
            sort: options.sort.map(|s| match s {
                SortOrder::Relevance => ModrinthSortOrder::Relevance,
                SortOrder::Downloads => ModrinthSortOrder::Downloads,
                SortOrder::Follows => ModrinthSortOrder::Follows,
                SortOrder::Newest => ModrinthSortOrder::Newest,
                SortOrder::Updated => ModrinthSortOrder::Updated,
            }),
            offset: options.offset,
            limit: options.limit,
            game_version: options.game_version.clone(),
            loader: options.loader.clone(),
            project_type: Some(match options.project_type.as_deref() {
                Some("modpack") => ModrinthProjectType::Modpack,
                Some("plugin") => ModrinthProjectType::Plugin,
                Some("resourcepack") => ModrinthProjectType::ResourcePack,
                Some("shader") => ModrinthProjectType::Shader,
                Some("datapack") => ModrinthProjectType::DataPack,
                _ => ModrinthProjectType::Mod,
            }),
        };

        let results = self.inner.search(&common_options).await?;

        Ok(results
            .into_iter()
            .filter(|p| p.server_side != "unsupported")
            .map(|p| Project {
                id: p.id,
                slug: p.slug,
                title: p.title,
                description: p.description,
                downloads: p.downloads,
                icon_url: p.icon_url,
                screenshot_urls: p.screenshot_urls,
                author: p.author,
                provider: ModProvider::Modrinth,
                categories: p.categories,
            })
            .collect())
    }

    pub async fn get_project(&self, id: &str) -> Result<Project> {
        let p = self.inner.get_project(id).await?;
        Ok(Project {
            id: p.id,
            slug: p.slug,
            title: p.title,
            description: p.description,
            downloads: p.downloads,
            icon_url: p.icon_url,
            screenshot_urls: p.screenshot_urls,
            author: p.author,
            provider: ModProvider::Modrinth,
            categories: p.categories,
        })
    }
}
