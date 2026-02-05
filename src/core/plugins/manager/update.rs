use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use crate::plugins::types::{PluginUpdate, PluginProvider};
use crate::plugins::modrinth::ModrinthClient;
use crate::plugins::spiget::SpigetClient;
use crate::plugins::hangar::HangarClient;
use crate::cache::CacheManager;
use super::list::list_installed_plugins;

/// Checks for updates for all installed plugins that have source information.
pub async fn check_for_updates(
    instance_path: impl AsRef<Path>,
    game_version: Option<&str>,
    loader: Option<&str>,
    cache: Arc<CacheManager>,
) -> Result<Vec<PluginUpdate>> {
    let installed = list_installed_plugins(&instance_path).await?;
    let mut updates = Vec::new();

    for plugin in installed {
        if let Some(source) = plugin.source {
            match source.provider {
                PluginProvider::Modrinth => {
                    let client = ModrinthClient::new(Arc::clone(&cache));
                    if let Ok(versions) = client.get_versions(&source.project_id, game_version, loader).await {
                        if let Some(latest) = versions.first() {
                            if Some(latest.id.clone()) != source.current_version_id {
                                updates.push(PluginUpdate {
                                    filename: plugin.filename.clone(),
                                    current_version: plugin.version.clone(),
                                    latest_version: latest.version_number.clone(),
                                    latest_version_id: latest.id.clone(),
                                    project_id: source.project_id.clone(),
                                    provider: source.provider,
                                });
                            }
                        }
                    }
                }
                PluginProvider::Spiget => {
                    let client = SpigetClient::new(Arc::clone(&cache));
                    if let Ok((latest_id, latest_name)) = client.get_latest_version(&source.project_id).await {
                        if Some(latest_id.clone()) != source.current_version_id {
                            updates.push(PluginUpdate {
                                filename: plugin.filename.clone(),
                                current_version: plugin.version.clone(),
                                latest_version: latest_name,
                                latest_version_id: latest_id,
                                project_id: source.project_id.clone(),
                                provider: source.provider,
                            });
                        }
                    }
                }
                PluginProvider::Hangar => {
                    let client = HangarClient::new(Arc::clone(&cache));
                    if let Ok(versions) = client.get_versions(&source.project_id, game_version, loader).await {
                        if let Some(latest) = versions.first() {
                            if Some(latest.id.clone()) != source.current_version_id {
                                updates.push(PluginUpdate {
                                    filename: plugin.filename.clone(),
                                    current_version: plugin.version.clone(),
                                    latest_version: latest.version_number.clone(),
                                    latest_version_id: latest.id.clone(),
                                    project_id: source.project_id.clone(),
                                    provider: source.provider,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(updates)
}
