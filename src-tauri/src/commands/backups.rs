use mc_server_wrapper_core::backup::{BackupManager, BackupInfo};
use mc_server_wrapper_core::instance::InstanceManager;
use tauri::{State, Window, Emitter};
use std::sync::Arc;
use uuid::Uuid;
use serde::Serialize;
use super::{CommandResult, AppError};

#[derive(Clone, Serialize)]
struct BackupProgress {
    instance_id: String,
    current: u64,
    total: u64,
    message: String,
}

#[tauri::command]
pub async fn list_backups(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_id: String,
) -> CommandResult<Vec<BackupInfo>> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    backup_manager.list_backups(id).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn create_backup(
    window: Window,
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    name: String,
) -> CommandResult<BackupInfo> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;

    let instance_id_clone = instance_id.clone();
    let window_clone = window.clone();

    backup_manager.create_backup(id, &instance.path, &name, move |current, total| {
        let _ = window_clone.emit("backup-progress", BackupProgress {
            instance_id: instance_id_clone.clone(),
            current,
            total,
            message: format!("Backing up files ({}/{})", current, total),
        });
    }).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn delete_backup(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_id: String,
    backup_name: String,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    backup_manager.delete_backup(id, &backup_name).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn restore_backup(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    backup_name: String,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;

    backup_manager.restore_backup(id, &backup_name, &instance.path).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn open_backup(
    backup_manager: State<'_, Arc<BackupManager>>,
    instance_id: String,
    backup_name: String,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let backups = backup_manager.list_backups(id).await.map_err(AppError::from)?;
    
    let backup = backups.into_iter()
        .find(|b| b.name == backup_name)
        .ok_or_else(|| AppError::NotFound("Backup not found".to_string()))?;

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // Use 'start' via 'cmd /c' to open the file with its default associated program
        Command::new("cmd")
            .args(["/C", "start", "", &backup.path.to_string_lossy()])
            .spawn()
            .map_err(AppError::from)?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&backup.path)
            .spawn()
            .map_err(AppError::from)?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&backup.path)
            .spawn()
            .map_err(AppError::from)?;
    }

    Ok(())
}
