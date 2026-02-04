use mc_server_wrapper_core::instance::{InstanceManager, InstanceMetadata};
use mc_server_wrapper_core::manager::ServerManager;
use tauri::{State, Emitter};
use std::sync::Arc;
use uuid::Uuid;
use super::super::{AppState, server::{ensure_server_logs_forwarded, LogPayload}, CommandResult, AppError};

#[tauri::command]
pub async fn list_instances(instance_manager: State<'_, Arc<InstanceManager>>) -> CommandResult<Vec<InstanceMetadata>> {
    instance_manager.list_instances().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn create_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    name: String,
    version: String,
) -> CommandResult<InstanceMetadata> {
    instance_manager.create_instance(&name, &version).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn check_instance_name_exists(
    instance_manager: State<'_, Arc<InstanceManager>>,
    name: String,
) -> CommandResult<bool> {
    let instance = instance_manager.get_instance_by_name(&name).await.map_err(AppError::from)?;
    Ok(instance.is_some())
}

#[tauri::command]
pub async fn delete_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    app_state: State<'_, AppState>,
    instance_id: String,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    
    // Remove from subscribed servers so if a new instance is created with same ID (unlikely) it can be re-subscribed
    let mut subscribed = app_state.subscribed_servers.lock().await;
    subscribed.remove(&id);
    drop(subscribed);

    instance_manager.delete_instance(id).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn delete_instance_by_name(
    instance_manager: State<'_, Arc<InstanceManager>>,
    name: String,
) -> CommandResult<()> {
    instance_manager.delete_instance_by_name(&name).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn clone_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    new_name: String,
) -> CommandResult<mc_server_wrapper_core::instance::InstanceMetadata> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    instance_manager.clone_instance(id, &new_name).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn open_instance_folder(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    if let Some(instance) = instance_manager.get_instance(id).await.map_err(AppError::from)? {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("explorer")
                .arg(instance.path)
                .spawn()
                .map_err(AppError::from)?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(instance.path)
                .spawn()
                .map_err(AppError::from)?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(instance.path)
                .spawn()
                .map_err(AppError::from)?;
        }
        Ok(())
    } else {
        Err(AppError::NotFound("Instance not found".to_string()))
    }
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
    start_after_creation: bool,
) -> CommandResult<mc_server_wrapper_core::instance::InstanceMetadata> {
    let instance = server_manager.create_instance_full(&name, &version, mod_loader, loader_version).await.map_err(AppError::from)?;
    
    // Auto-start or prepare the server
    let instance_id = instance.id.to_string();
    let id = instance.id;
    
    // We run start_server/prepare_server in a separate task so we can return the instance metadata immediately
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

        if start_after_creation {
            if let Err(e) = server_manager_clone.start_server(id).await {
                let _ = app_handle_clone.emit("server-log", LogPayload {
                    instance_id: instance_id_clone,
                    line: format!("Error starting server: {}", e),
                });
            }
        } else {
            if let Err(e) = server_manager_clone.prepare_server(id).await {
                let _ = app_handle_clone.emit("server-log", LogPayload {
                    instance_id: instance_id_clone,
                    line: format!("Error preparing server: {}", e),
                });
            }
        }
    });

    Ok(instance)
}
