use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::server::{ServerStatus, ResourceUsage, ServerHandle};
use mc_server_wrapper_core::instance::InstanceManager;
use tauri::{State, Emitter};
use std::sync::Arc;
use uuid::Uuid;
use super::AppState;

#[derive(Clone, serde::Serialize)]
pub struct LogPayload {
    pub instance_id: String,
    pub line: String,
}

#[derive(Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub instance_id: String,
    pub current: u64,
    pub total: u64,
    pub message: String,
}

pub async fn ensure_server_logs_forwarded(
    app_state: &AppState,
    server: Arc<ServerHandle>,
    app_handle: tauri::AppHandle,
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let mut subscribed = app_state.subscribed_servers.lock().await;
    
    if !subscribed.contains(&id) {
        subscribed.insert(id);
        
        let mut rx = server.subscribe_logs();
        let mut rx_progress = server.subscribe_progress();
        let instance_id_clone = instance_id.clone();
        let app_handle_clone = app_handle.clone();
        
        tauri::async_runtime::spawn(async move {
            while let Ok(line) = rx.recv().await {
                let _ = app_handle_clone.emit("server-log", LogPayload {
                    instance_id: instance_id_clone.clone(),
                    line,
                });
            }
        });

        let instance_id_clone2 = instance_id.clone();
        let app_handle_clone2 = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            while let Ok(progress) = rx_progress.recv().await {
                let _ = app_handle_clone2.emit("download-progress", ProgressPayload {
                    instance_id: instance_id_clone2.clone(),
                    current: progress.current,
                    total: progress.total,
                    message: progress.message,
                });
            }
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn start_server(
    server_manager: State<'_, Arc<ServerManager>>,
    app_state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    
    // Get or create handle early so we can subscribe to logs during installation
    let server = server_manager.get_or_create_server(id).await.map_err(|e| e.to_string())?;
    
    ensure_server_logs_forwarded(&app_state, server, app_handle.clone(), instance_id.clone()).await?;

    // Start the server in a separate task so the UI can receive logs immediately
    // especially during the installation phase which might take time.
    let server_manager_inner = server_manager.inner().clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = server_manager_inner.start_server(id).await {
            let _ = app_handle.emit("server-log", LogPayload {
                instance_id: instance_id.clone(),
                line: format!("Error starting server: {}", e),
            });
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_server(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    server_manager.stop_server(id).await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub async fn get_server_status(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
) -> Result<String, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(server) = server_manager.get_server(id).await {
        Ok(server.get_status().await.to_string())
    } else {
        Ok(ServerStatus::Stopped.to_string())
    }
}

#[tauri::command]
pub async fn get_server_usage(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
) -> Result<ResourceUsage, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(server) = server_manager.get_server(id).await {
        Ok(server.get_usage().await)
    } else {
        Ok(ResourceUsage::default())
    }
}

#[tauri::command]
pub async fn send_command(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
    command: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(server) = server_manager.get_server(id).await {
        server.send_command(&command).await.map_err(|e: anyhow::Error| e.to_string())
    } else {
        Err("Server is not running".to_string())
    }
}

#[tauri::command]
pub async fn read_latest_log(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<String, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(instance) = instance_manager.get_instance(id).await.map_err(|e: anyhow::Error| e.to_string())? {
        let log_path = instance.path.join("logs").join("latest.log");
        if log_path.exists() {
            tokio::fs::read_to_string(log_path).await.map_err(|e| e.to_string())
        } else {
            Ok("".to_string())
        }
    } else {
        Err("Instance not found".to_string())
    }
}
