use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::plugins::{self, InstalledPlugin};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;
use super::super::{CommandResult, AppError};

#[tauri::command]
pub async fn list_installed_plugins(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> CommandResult<Vec<InstalledPlugin>> {
    let instances = instance_manager.list_instances().await.map_err(AppError::from)?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    plugins::list_installed_plugins(&instance.path).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn toggle_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    enable: bool,
) -> CommandResult<()> {
    let instances = instance_manager.list_instances().await.map_err(AppError::from)?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    plugins::toggle_plugin(&instance.path, filename, enable).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn bulk_toggle_plugins(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    enable: bool,
) -> CommandResult<()> {
    let instances = instance_manager.list_instances().await.map_err(AppError::from)?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    plugins::bulk_toggle_plugins(&instance.path, filenames, enable).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn uninstall_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    delete_config: bool,
) -> CommandResult<()> {
    let instances = instance_manager.list_instances().await.map_err(AppError::from)?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    plugins::uninstall_plugin(&instance.path, filename, delete_config).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn bulk_uninstall_plugins(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    delete_config: bool,
) -> CommandResult<()> {
    let instances = instance_manager.list_instances().await.map_err(AppError::from)?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    plugins::bulk_uninstall_plugins(&instance.path, filenames, delete_config).await.map_err(AppError::from)
}
