use super::ModrinthClient;
use crate::modrinth::{ModrinthProjectType, ModrinthSearchOptions, ModrinthSortOrder};
use crate::plugins::types::{
    PluginProvider, Project, ResolvedDependency, SearchOptions, SortOrder,
};
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
            project_type: Some(ModrinthProjectType::Plugin),
        };

        let results = self.inner.search(&common_options).await?;

        Ok(results
            .into_iter()
            .map(|p| Project {
                id: p.id,
                slug: p.slug,
                title: p.title,
                description: p.description,
                downloads: p.downloads,
                icon_url: p.icon_url,
                screenshot_urls: p.screenshot_urls,
                author: p.author,
                provider: PluginProvider::Modrinth,
                categories: p.categories,
            })
            .collect())
    }

    pub async fn get_dependencies(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ResolvedDependency>> {
        let deps = self
            .inner
            .get_dependencies(
                project_id,
                game_version,
                loader,
                Some(ModrinthProjectType::Plugin),
            )
            .await?;

        Ok(deps
            .into_iter()
            .map(|(p, dep_type)| ResolvedDependency {
                project: Project {
                    id: p.id,
                    slug: p.slug,
                    title: p.title,
                    description: p.description,
                    downloads: p.downloads,
                    icon_url: p.icon_url,
                    screenshot_urls: p.screenshot_urls,
                    author: p.author,
                    provider: PluginProvider::Modrinth,
                    categories: p.categories,
                },
                dependency_type: dep_type,
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
            provider: PluginProvider::Modrinth,
            categories: p.categories,
        })
    }
}
