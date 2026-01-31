use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::server::{ServerStatus, ResourceUsage};
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

#[tauri::command]
async fn start_server(
    server_manager: State<'_, Arc<ServerManager>>,
    app_handle: tauri::AppHandle,
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    server_manager.start_server(id).await.map_err(|e: anyhow::Error| e.to_string())?;

    // Subscribe to logs
    if let Some(server) = server_manager.get_server(id).await {
        let mut rx = server.subscribe_logs();
        let instance_id_clone = instance_id.clone();
        
        tauri::async_runtime::spawn(async move {
            while let Ok(line) = rx.recv().await {
                let _ = app_handle.emit("server-log", LogPayload {
                    instance_id: instance_id_clone.clone(),
                    line,
                });
            }
        });
    }

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
    instance_id: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
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
    name: String,
    version: String,
    mod_loader: Option<String>,
    loader_version: Option<String>,
) -> Result<mc_server_wrapper_core::instance::InstanceMetadata, String> {
    server_manager.create_instance_full(&name, &version, mod_loader, loader_version).await.map_err(|e| e.to_string())
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

      // Background task to bridge logs to Tauri events
      tauri::async_runtime::spawn(async move {
          loop {
              // We need a way to know when a new server starts to subscribe to its logs
              // For now, let's just check all servers every few seconds and subscribe if not already
              // Actually, a better way would be for ServerManager to have a way to notify us.
              // But let's keep it simple for now and just subscribe when a server is started via the command.
              tokio::time::sleep(std::time::Duration::from_secs(1)).await;
          }
      });

      app.manage(instance_manager);
      app.manage(server_manager);
      
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
