use mc_server_wrapper_core::instance::{InstanceManager, InstanceSettings};
use mc_server_wrapper_core::manager::ServerManager;
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn update_instance_settings(
    instance_manager: State<'_, Arc<InstanceManager>>,
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
    name: Option<String>,
    settings: InstanceSettings,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    instance_manager.update_settings(id, name, settings).await.map_err(|e| e.to_string())?;
    
    // If the server is already loaded in memory, update its config
    let _ = server_manager.prepare_server(id).await;
    
    Ok(())
}

#[tauri::command]
pub async fn list_bat_files(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<Vec<String>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mut bat_files = Vec::new();
    let mut entries = tokio::fs::read_dir(&instance.path).await.map_err(|e| e.to_string())?;
    
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "bat" || ext == "sh" || ext == "ps1" {
                    if let Some(file_name) = path.file_name() {
                        bat_files.push(file_name.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    Ok(bat_files)
}

#[tauri::command]
pub async fn update_instance_jar(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    source_path: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let dest_path = instance.path.join("server.jar");
    tokio::fs::copy(source_path, dest_path).await.map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_startup_preview(
    _instance_id: String,
    settings: InstanceSettings,
) -> Result<String, String> {
    let mut line = settings.startup_line.clone();
    line = line.replace("{ram}", &settings.ram.to_string());
    line = line.replace("{unit}", &settings.ram_unit);
    
    Ok(line)
}
