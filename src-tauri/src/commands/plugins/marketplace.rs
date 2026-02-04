use mc_server_wrapper_core::plugins::{self, Project, PluginProvider, SearchOptions, ResolvedDependency};
use mc_server_wrapper_core::manager::ServerManager;
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;
use super::super::{CommandResult, AppError};

#[tauri::command]
pub async fn search_plugins(
    server_manager: State<'_, Arc<ServerManager>>,
    options: SearchOptions,
    provider: Option<PluginProvider>,
) -> CommandResult<Vec<Project>> {
    plugins::search_plugins(&options, provider, server_manager.get_cache()).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_plugin_dependencies(
    server_manager: State<'_, Arc<ServerManager>>,
    project_id: String,
    provider: PluginProvider,
) -> CommandResult<Vec<ResolvedDependency>> {
    plugins::get_plugin_dependencies(&project_id, provider, server_manager.get_cache()).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn install_plugin(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: Uuid,
    project_id: String,
    provider: PluginProvider,
    version_id: Option<String>,
) -> CommandResult<String> {
    let instances = server_manager.get_instance_manager().list_instances().await.map_err(AppError::from)?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    plugins::install_plugin(
        &instance.path, 
        &project_id, 
        provider, 
        version_id.as_deref(),
        Some(&instance.version),
        instance.mod_loader.as_deref(),
        server_manager.get_cache()
    )
        .await
        .map_err(AppError::from)
}
