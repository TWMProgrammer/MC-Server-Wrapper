use anyhow::Result;
use crate::mods::types::{ModProvider, SearchOptions, Project, ResolvedDependency};
use crate::mods::modrinth::ModrinthClient;
use crate::mods::curseforge::CurseForgeClient;

/// Searches for mods across multiple providers.
pub async fn search_mods(options: &SearchOptions, provider: Option<ModProvider>, curseforge_api_key: Option<String>) -> Result<Vec<Project>> {
    let mut results = Vec::new();

    match provider {
        Some(ModProvider::Modrinth) => {
            let client = ModrinthClient::new();
            results.extend(client.search(options).await?);
        }
        Some(ModProvider::CurseForge) => {
            let client = CurseForgeClient::new(curseforge_api_key);
            results.extend(client.search(options).await?);
        }
        None => {
            let modrinth = ModrinthClient::new();
            let curseforge = CurseForgeClient::new(curseforge_api_key);

            let (m_res, c_res): (Result<Vec<Project>>, Result<Vec<Project>>) = tokio::join!(
                modrinth.search(options),
                curseforge.search(options)
            );

            if let Ok(res) = m_res { results.extend(res); }
            if let Ok(res) = c_res { results.extend(res); }
        }
    }

    Ok(results)
}

/// Gets dependencies for a mod.
pub async fn get_mod_dependencies(
    project_id: &str,
    provider: ModProvider,
    game_version: Option<&str>,
    loader: Option<&str>,
    curseforge_api_key: Option<String>,
) -> Result<Vec<ResolvedDependency>> {
    match provider {
        ModProvider::Modrinth => {
            let client = ModrinthClient::new();
            client.get_dependencies(project_id, game_version, loader).await
        }
        ModProvider::CurseForge => {
            let client = CurseForgeClient::new(curseforge_api_key);
            client.get_dependencies(project_id, game_version, loader).await
        }
    }
}
