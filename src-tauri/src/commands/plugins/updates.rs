use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::plugins::{self, PluginProvider, PluginUpdate};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn check_for_plugin_updates(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> Result<Vec<PluginUpdate>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::check_for_updates(
        &instance.path,
        Some(instance.version.as_str()),
        instance.mod_loader.as_deref(),
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    project_id: String,
    provider: PluginProvider,
    latest_version_id: String,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::update_plugin(&instance.path, filename, project_id, provider, latest_version_id).await.map_err(|e| e.to_string())
}
