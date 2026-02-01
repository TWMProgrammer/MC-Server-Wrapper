use mc_server_wrapper_core::instance::{InstanceManager, InstanceSettings};
use mc_server_wrapper_core::manager::ServerManager;
use tauri::{State, Emitter};
use std::sync::Arc;
use uuid::Uuid;
use super::{AppState, server::{ensure_server_logs_forwarded, LogPayload}};

#[tauri::command]
pub async fn list_instances(instance_manager: State<'_, Arc<InstanceManager>>) -> Result<Vec<mc_server_wrapper_core::instance::InstanceMetadata>, String> {
    instance_manager.list_instances().await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn create_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    name: String,
    version: String,
) -> Result<mc_server_wrapper_core::instance::InstanceMetadata, String> {
    instance_manager.create_instance(&name, &version).await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn delete_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    app_state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    
    // Remove from subscribed servers so if a new instance is created with same ID (unlikely) it can be re-subscribed
    let mut subscribed = app_state.subscribed_servers.lock().await;
    subscribed.remove(&id);
    drop(subscribed);

    instance_manager.delete_instance(id).await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn clone_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    new_name: String,
) -> Result<mc_server_wrapper_core::instance::InstanceMetadata, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    instance_manager.clone_instance(id, &new_name).await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn open_instance_folder(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(instance) = instance_manager.get_instance(id).await.map_err(|e: anyhow::Error| e.to_string())? {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(instance.path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(instance.path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(instance.path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("Instance not found".to_string())
    }
}

#[tauri::command]
pub async fn get_minecraft_versions(server_manager: State<'_, Arc<ServerManager>>) -> Result<mc_server_wrapper_core::downloader::VersionManifest, String> {
    server_manager.get_downloader().fetch_manifest().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_mod_loaders(server_manager: State<'_, Arc<ServerManager>>, mc_version: String) -> Result<Vec<mc_server_wrapper_core::mod_loaders::ModLoader>, String> {
    server_manager.get_mod_loader_client().get_available_loaders(&mc_version).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_instance_full(
    server_manager: State<'_, Arc<ServerManager>>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    name: String,
    version: String,
    mod_loader: Option<String>,
    loader_version: Option<String>,
) -> Result<mc_server_wrapper_core::instance::InstanceMetadata, String> {
    let instance = server_manager.create_instance_full(&name, &version, mod_loader, loader_version).await.map_err(|e| e.to_string())?;
    
    // Auto-start the server
    let instance_id = instance.id.to_string();
    let id = instance.id;
    
    // We run start_server in a separate task so we can return the instance metadata immediately
    // while the server starts (which might involve downloading)
    let server_manager_clone = server_manager.inner().clone();
    let app_state_clone = app_state.inner().clone();
    let app_handle_clone = app_handle.clone();
    let instance_id_clone = instance_id.clone();
    
    tauri::async_runtime::spawn(async move {
        // Get or create handle early so we can subscribe to logs during installation
        let server = match server_manager_clone.get_or_create_server(id).await {
            Ok(s) => s,
            Err(e) => {
                let _ = app_handle_clone.emit("server-log", LogPayload {
                    instance_id: instance_id_clone,
                    line: format!("Error preparing server: {}", e),
                });
                return;
            }
        };

        // Ensure logs are forwarded
        if let Err(e) = ensure_server_logs_forwarded(&app_state_clone, server, app_handle_clone.clone(), instance_id_clone.clone()).await {
            let _ = app_handle_clone.emit("server-log", LogPayload {
                instance_id: instance_id_clone.clone(),
                line: format!("Error setting up log forwarding: {}", e),
            });
        }

        if let Err(e) = server_manager_clone.start_server(id).await {
            let _ = app_handle_clone.emit("server-log", LogPayload {
                instance_id: instance_id_clone,
                line: format!("Error starting server: {}", e),
            });
        }
    });

    Ok(instance)
}

#[tauri::command]
pub async fn update_instance_settings(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    name: Option<String>,
    settings: InstanceSettings,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    instance_manager.update_settings(id, name, settings).await.map_err(|e| e.to_string())
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
