use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::utils::safe_join;
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;
use super::{CommandResult, AppError};

#[tauri::command]
pub async fn read_text_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
) -> CommandResult<String> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    let file_path = safe_join(&instance.path, &rel_path).map_err(AppError::from)?;
    if !file_path.exists() {
        return Ok("".to_string());
    }

    tokio::fs::read_to_string(file_path).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn save_text_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    content: String,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    let file_path = safe_join(&instance.path, &rel_path).map_err(AppError::from)?;
    
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(AppError::from)?;
    }

    tokio::fs::write(file_path, content).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn open_file_in_editor(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    let file_path = safe_join(&instance.path, &rel_path).map_err(AppError::from)?;
    if !file_path.exists() {
        return Err(AppError::NotFound("File does not exist".to_string()));
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, 'explorer' opens the parent folder if the path is a file.
        // We use 'powershell' with 'Start-Process' to correctly open the file with its default application.
        std::process::Command::new("powershell")
            .arg("-Command")
            .arg(format!("Start-Process '{}'", file_path.display()))
            .spawn()
            .map_err(AppError::from)?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(file_path)
            .spawn()
            .map_err(AppError::from)?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(file_path)
            .spawn()
            .map_err(AppError::from)?;
    }

    Ok(())
}
