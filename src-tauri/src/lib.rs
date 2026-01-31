use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::server::{ServerStatus, ResourceUsage, ServerHandle};
use tauri::{State, Manager, Emitter};
use std::sync::Arc;
use uuid::Uuid;

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
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
