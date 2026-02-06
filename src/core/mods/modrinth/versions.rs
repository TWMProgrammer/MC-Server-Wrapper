use super::ModrinthClient;
use crate::mods::types::{
    Dependency, ModProvider, Project, ProjectFile, ProjectVersion, ResolvedDependency,
};
use anyhow::Result;

impl ModrinthClient {
    pub async fn get_dependencies(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ResolvedDependency>> {
        let deps = self
            .inner
            .get_dependencies(project_id, game_version, loader)
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
                    provider: ModProvider::Modrinth,
                    categories: p.categories,
                },
                dependency_type: dep_type,
            })
            .collect())
    }

    pub async fn get_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let versions = self
            .inner
            .get_versions(project_id, game_version, loader)
            .await?;

        Ok(versions
            .into_iter()
            .map(|v| ProjectVersion {
                id: v.id,
                project_id: v.project_id,
                version_number: v.version_number,
                files: v
                    .files
                    .into_iter()
                    .map(|f| ProjectFile {
                        url: f.url,
                        filename: f.filename,
                        primary: f.primary,
                        size: f.size,
                    })
                    .collect(),
                loaders: v.loaders,
                game_versions: v.game_versions,
                dependencies: v
                    .dependencies
                    .into_iter()
                    .map(|d| Dependency {
                        project_id: d.project_id,
                        version_id: d.version_id,
                        dependency_type: d.dependency_type,
                    })
                    .collect(),
            })
            .collect())
    }
}
