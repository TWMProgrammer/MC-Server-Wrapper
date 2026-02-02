use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::mods::{self, InstalledMod};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn list_installed_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> Result<Vec<InstalledMod>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::list_installed_mods(&instance.path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    enable: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::toggle_mod(&instance.path, filename, enable).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_toggle_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    enable: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::bulk_toggle_mods(&instance.path, filenames, enable).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    delete_config: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::uninstall_mod(&instance.path, filename, delete_config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_uninstall_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    delete_config: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::bulk_uninstall_mods(&instance.path, filenames, delete_config).await.map_err(|e| e.to_string())
}
