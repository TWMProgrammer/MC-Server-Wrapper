use anyhow::Result;
use super::ModrinthClient;
use crate::plugins::types::ProjectVersion;

impl ModrinthClient {
    pub async fn get_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let cache_key = format!("modrinth_versions_{}_v:{:?}_lo:{:?}", project_id, game_version, loader);
        if let Ok(Some(cached)) = self.cache.get::<Vec<ProjectVersion>>(&cache_key).await {
            return Ok(cached);
        }

        let mut url = format!("https://api.modrinth.com/v2/project/{}/version", project_id);
        
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

        let versions = self.client.get(&url).send().await?.json::<Vec<ProjectVersion>>().await?;
        let _ = self.cache.set(cache_key, versions.clone()).await;
        Ok(versions)
    }
}
