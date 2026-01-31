use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::server::{ServerStatus, ResourceUsage, ServerHandle};
use mc_server_wrapper_core::players;
use mc_server_wrapper_core::config_files;
use tauri::{State, Manager, Emitter};
use std::sync::Arc;
use uuid::Uuid;
use chrono;

#[tauri::command]
async fn read_text_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
) -> Result<String, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let file_path = instance.path.join(&rel_path);
    if !file_path.exists() {
        return Ok("".to_string());
    }

    tokio::fs::read_to_string(file_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_text_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    content: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let file_path = instance.path.join(&rel_path);
    
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }

    tokio::fs::write(file_path, content).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_instances(instance_manager: State<'_, Arc<InstanceManager>>) -> Result<Vec<mc_server_wrapper_core::instance::InstanceMetadata>, String> {
    instance_manager.list_instances().await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
async fn create_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    name: String,
    version: String,
) -> Result<mc_server_wrapper_core::instance::InstanceMetadata, String> {
    instance_manager.create_instance(&name, &version).await.map_err(|e: anyhow::Error| e.to_string())
}

use std::collections::HashSet;
use tokio::sync::Mutex as TokioMutex;

#[derive(Clone)]
struct AppState {
    subscribed_servers: Arc<TokioMutex<HashSet<Uuid>>>,
}

async fn ensure_server_logs_forwarded(
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
async fn start_server(
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
async fn stop_server(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    server_manager.stop_server(id).await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
async fn get_server_status(
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
async fn get_server_usage(
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
async fn send_command(
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
async fn delete_instance(
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
async fn clone_instance(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    new_name: String,
) -> Result<mc_server_wrapper_core::instance::InstanceMetadata, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    instance_manager.clone_instance(id, &new_name).await.map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
async fn open_instance_folder(
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
async fn open_player_list_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    list_type: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(instance) = instance_manager.get_instance(id).await.map_err(|e: anyhow::Error| e.to_string())? {
        let file_name = match list_type.as_str() {
            "whitelist" => "whitelist.json",
            "ops" => "ops.json",
            "banned-players" => "banned-players.json",
            "banned-ips" => "banned-ips.json",
            _ => return Err("Invalid list type".to_string()),
        };
        let file_path = instance.path.join(file_name);
        
        // Create the file if it doesn't exist, so the editor can open it
        if !file_path.exists() {
            tokio::fs::write(&file_path, "[]").await.map_err(|e| e.to_string())?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!("Start-Process '{}'", file_path.to_string_lossy()))
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(file_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(file_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("Instance not found".to_string())
    }
}

#[tauri::command]
async fn read_latest_log(
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

#[tauri::command]
async fn get_online_players(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
) -> Result<Vec<String>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(server) = server_manager.get_server(id).await {
        Ok(server.get_online_players().await)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
async fn get_players(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<players::AllPlayerLists, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let whitelist = players::read_whitelist(&instance.path).await.map_err(|e| e.to_string())?;
    let ops = players::read_ops(&instance.path).await.map_err(|e| e.to_string())?;
    let banned_players = players::read_banned_players(&instance.path).await.map_err(|e| e.to_string())?;
    let banned_ips = players::read_banned_ips(&instance.path).await.map_err(|e| e.to_string())?;
    let user_cache = players::read_usercache(&instance.path).await.map_err(|e| e.to_string())?;
    
    Ok(players::AllPlayerLists {
        whitelist,
        ops,
        banned_players,
        banned_ips,
        user_cache,
    })
}

#[tauri::command]
async fn add_player(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    list_type: String,
    username: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    let (uuid, name) = players::fetch_player_uuid(&username).await.map_err(|e| e.to_string())?;

    match list_type.as_str() {
        "whitelist" => {
            let mut list = players::read_whitelist(&instance.path).await.map_err(|e| e.to_string())?;
            if !list.iter().any(|p| p.uuid == uuid) {
                list.push(players::PlayerEntry { uuid, name });
                players::write_whitelist(&instance.path, &list).await.map_err(|e| e.to_string())?;
            }
        },
        "ops" => {
            let mut list = players::read_ops(&instance.path).await.map_err(|e| e.to_string())?;
            if !list.iter().any(|p| p.uuid == uuid) {
                list.push(players::OpEntry { uuid, name, level: 4, bypasses_player_limit: false });
                players::write_ops(&instance.path, &list).await.map_err(|e| e.to_string())?;
            }
        },
        "banned-players" => {
            let mut list = players::read_banned_players(&instance.path).await.map_err(|e| e.to_string())?;
            if !list.iter().any(|p| p.uuid == uuid) {
                list.push(players::BannedPlayerEntry {
                    uuid,
                    name,
                    created: chrono::Utc::now().to_rfc3339(),
                    source: "Server Wrapper".to_string(),
                    expires: "forever".to_string(),
                    reason: "Banned by admin".to_string(),
                });
                players::write_banned_players(&instance.path, &list).await.map_err(|e| e.to_string())?;
            }
        },
        _ => return Err("Invalid list type".to_string()),
    }
    Ok(())
}

#[tauri::command]
async fn add_banned_ip(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    ip: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    let mut list = players::read_banned_ips(&instance.path).await.map_err(|e| e.to_string())?;
    if !list.iter().any(|p| p.ip == ip) {
        list.push(players::BannedIpEntry {
            ip,
            created: chrono::Utc::now().to_rfc3339(),
            source: "Server Wrapper".to_string(),
            expires: "forever".to_string(),
            reason: "Banned by admin".to_string(),
        });
        players::write_banned_ips(&instance.path, &list).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn remove_player(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    list_type: String,
    identifier: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    match list_type.as_str() {
        "whitelist" => {
            let mut list = players::read_whitelist(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.uuid != identifier);
            players::write_whitelist(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        "ops" => {
            let mut list = players::read_ops(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.uuid != identifier);
            players::write_ops(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        "banned-players" => {
            let mut list = players::read_banned_players(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.uuid != identifier);
            players::write_banned_players(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        "banned-ips" => {
            let mut list = players::read_banned_ips(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.ip != identifier);
            players::write_banned_ips(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        _ => return Err("Invalid list type".to_string()),
    }
    Ok(())
}

#[tauri::command]
async fn get_config_value(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
) -> Result<serde_json::Value, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::read_config_value(&instance.path, &rel_path, format).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_config_value(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
    value: serde_json::Value,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::save_config_value(&instance.path, &rel_path, format, value).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_available_configs(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<Vec<config_files::ConfigFile>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    Ok(config_files::list_available_configs(&instance.path, instance.mod_loader.as_deref()).await)
}

#[tauri::command]
async fn get_config_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
) -> Result<std::collections::HashMap<String, String>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::read_config_file(&instance.path, &rel_path, format).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_config_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
    properties: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::save_config_file(&instance.path, &rel_path, format, properties).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_server_properties(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::read_config_file(&instance.path, "server.properties", config_files::ConfigFormat::Properties).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_server_properties(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    properties: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::save_config_file(&instance.path, "server.properties", config_files::ConfigFormat::Properties, properties).await.map_err(|e| e.to_string())
}

#[derive(Clone, serde::Serialize)]
struct LogPayload {
    instance_id: String,
    line: String,
}

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    instance_id: String,
    current: u64,
    total: u64,
    message: String,
}

#[tauri::command]
async fn get_minecraft_versions(server_manager: State<'_, Arc<ServerManager>>) -> Result<mc_server_wrapper_core::downloader::VersionManifest, String> {
    server_manager.get_downloader().fetch_manifest().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_mod_loaders(server_manager: State<'_, Arc<ServerManager>>, mc_version: String) -> Result<Vec<mc_server_wrapper_core::mod_loaders::ModLoader>, String> {
    server_manager.get_mod_loader_client().get_available_loaders(&mc_version).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn create_instance_full(
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      // Set window size to 70% of screen resolution
      if let Some(window) = app.get_webview_window("main") {
          if let Ok(Some(monitor)) = window.primary_monitor() {
              let size = monitor.size();
              let width = (size.width as f64 * 0.7) as u32;
              let height = (size.height as f64 * 0.7) as u32;
              let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                  width,
                  height,
              }));
              let _ = window.center();
          }
      }

      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Initialize Directories next to the executable
      let exe_path = std::env::current_exe()
          .map(|p| p.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| p))
          .expect("failed to get exe directory");
      let app_dirs = tauri::async_runtime::block_on(async {
          mc_server_wrapper_core::init::init_directories(&exe_path).await.expect("failed to initialize directories")
      });
      
      // Initialize InstanceManager using the 'server' directory
      let instance_manager = Arc::new(tauri::async_runtime::block_on(async {
          InstanceManager::new(app_dirs.server).await.expect("failed to initialize instance manager")
      }));

      let server_manager = Arc::new(ServerManager::new(Arc::clone(&instance_manager)));

      app.manage(instance_manager);
      app.manage(server_manager);
      app.manage(AppState {
          subscribed_servers: Arc::new(TokioMutex::new(HashSet::new())),
      });
      
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        read_text_file,
        save_text_file,
        list_instances,
        create_instance,
        start_server,
        stop_server,
        get_server_status,
        get_server_usage,
        send_command,
        get_minecraft_versions,
        get_mod_loaders,
        create_instance_full,
        delete_instance,
        clone_instance,
        open_instance_folder,
        open_player_list_file,
        read_latest_log,
        get_players,
        get_online_players,
        add_player,
        add_banned_ip,
        remove_player,
        get_server_properties,
        save_server_properties,
        get_available_configs,
        get_config_file,
        save_config_file,
        get_config_value,
        save_config_value,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
