use anyhow::Result;
use super::ModrinthClient;
use crate::plugins::types::{ProjectVersion, ProjectFile};

impl ModrinthClient {
    pub async fn get_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let versions = self.inner.get_versions(project_id, game_version, loader).await?;

        Ok(versions
            .into_iter()
            .map(|v| ProjectVersion {
                id: v.id,
                project_id: v.project_id,
                version_number: v.version_number,
                files: v.files.into_iter().map(|f| ProjectFile {
                    url: f.url,
                    filename: f.filename,
                    primary: f.primary,
                    size: f.size,
                    sha1: f.hashes.and_then(|h| h.sha1),
                }).collect(),
                loaders: v.loaders,
                game_versions: v.game_versions,
            })
            .collect())
    }
}
