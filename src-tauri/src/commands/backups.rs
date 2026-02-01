use mc_server_wrapper_core::backup::{BackupManager, BackupInfo};
use mc_server_wrapper_core::instance::InstanceManager;
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn list_backups(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_id: String,
) -> Result<Vec<BackupInfo>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    backup_manager.list_backups(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_backup(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    name: String,
) -> Result<BackupInfo, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    backup_manager.create_backup(id, &instance.path, &name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_backup(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_id: String,
    backup_name: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    backup_manager.delete_backup(id, &backup_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_backup(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    backup_name: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    backup_manager.restore_backup(id, &backup_name, &instance.path).await.map_err(|e| e.to_string())
}
