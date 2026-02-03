use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::plugins::{self, Project, PluginProvider, SearchOptions, ResolvedDependency};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn search_plugins(
    options: SearchOptions,
    provider: Option<PluginProvider>,
) -> Result<Vec<Project>, String> {
    plugins::search_plugins(&options, provider).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_plugin_dependencies(
    project_id: String,
    provider: PluginProvider,
) -> Result<Vec<ResolvedDependency>, String> {
    plugins::get_plugin_dependencies(&project_id, provider).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    project_id: String,
    provider: PluginProvider,
    version_id: Option<String>,
) -> Result<String, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::install_plugin(
        &instance.path, 
        &project_id, 
        provider, 
        version_id.as_deref(),
        Some(&instance.version),
        instance.mod_loader.as_deref()
    )
        .await
        .map_err(|e| e.to_string())
}
