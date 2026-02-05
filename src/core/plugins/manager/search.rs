use std::sync::Arc;
use anyhow::Result;
use crate::plugins::types::{Project, SearchOptions, PluginProvider, ResolvedDependency};
use crate::plugins::modrinth::ModrinthClient;
use crate::plugins::spiget::SpigetClient;
use crate::plugins::hangar::HangarClient;
use crate::cache::CacheManager;

/// Searches for plugins across multiple providers.
pub async fn search_plugins(options: &SearchOptions, provider: Option<PluginProvider>, cache: Arc<CacheManager>) -> Result<Vec<Project>> {
    let mut results = Vec::new();

    match provider {
        Some(PluginProvider::Modrinth) => {
            let client = ModrinthClient::new(cache);
            results.extend(client.search(options).await?);
        }
        Some(PluginProvider::Spiget) => {
            let client = SpigetClient::new(cache);
            results.extend(client.search(options).await?);
        }
        Some(PluginProvider::Hangar) => {
            let client = HangarClient::new(cache);
            results.extend(client.search(options).await?);
        }
        None => {
            // Search all providers
            let modrinth = ModrinthClient::new(Arc::clone(&cache));
            let spiget = SpigetClient::new(Arc::clone(&cache));
            let hangar = HangarClient::new(cache);

            let (m_res, s_res, h_res) = tokio::join!(
                modrinth.search(options),
                spiget.search(options),
                hangar.search(options)
            );

            if let Ok(res) = m_res { results.extend(res); }
            if let Ok(res) = s_res { results.extend(res); }
            if let Ok(res) = h_res { results.extend(res); }
        }
    }

    Ok(results)
}

/// Gets dependencies for a plugin.
pub async fn get_plugin_dependencies(project_id: &str, provider: PluginProvider, cache: Arc<CacheManager>) -> Result<Vec<ResolvedDependency>> {
    match provider {
        PluginProvider::Modrinth => {
            let client = ModrinthClient::new(cache);
            client.get_dependencies(project_id).await
        }
        PluginProvider::Spiget => {
            let client = SpigetClient::new(cache);
            client.get_dependencies(project_id).await
        }
        PluginProvider::Hangar => {
            let client = HangarClient::new(cache);
            client.get_dependencies(project_id).await
        }
    }
}
