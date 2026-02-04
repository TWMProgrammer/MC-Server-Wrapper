use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::plugins::{self, PluginProvider, PluginUpdate};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn check_for_plugin_updates(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: Uuid,
) -> Result<Vec<PluginUpdate>, String> {
    let instances = server_manager.get_instance_manager().list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::check_for_updates(
        &instance.path,
        Some(instance.version.as_str()),
        instance.mod_loader.as_deref(),
        server_manager.get_cache()
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_plugin(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: Uuid,
    filename: String,
    project_id: String,
    provider: PluginProvider,
    latest_version_id: String,
) -> Result<(), String> {
    let instances = server_manager.get_instance_manager().list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::update_plugin(
        &instance.path, 
        filename, 
        project_id, 
        provider, 
        latest_version_id,
        server_manager.get_cache()
    ).await.map_err(|e| e.to_string())
}
