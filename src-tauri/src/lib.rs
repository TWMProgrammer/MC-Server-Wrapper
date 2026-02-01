mod commands;

use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use std::collections::HashSet;
use commands::AppState;

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
        commands::files::read_text_file,
        commands::files::save_text_file,
        commands::files::open_file_in_editor,
        commands::instance::list_instances,
        commands::instance::create_instance,
        commands::instance::delete_instance,
        commands::instance::clone_instance,
        commands::instance::open_instance_folder,
        commands::instance::get_minecraft_versions,
        commands::instance::get_mod_loaders,
        commands::instance::create_instance_full,
        commands::server::start_server,
        commands::server::stop_server,
        commands::server::get_server_status,
        commands::server::get_server_usage,
        commands::server::send_command,
        commands::server::read_latest_log,
        commands::players::open_player_list_file,
        commands::players::get_players,
        commands::players::get_online_players,
        commands::players::add_player,
        commands::players::add_banned_ip,
        commands::players::remove_player,
        commands::config::get_server_properties,
        commands::config::save_server_properties,
        commands::config::get_available_configs,
        commands::config::get_config_file,
        commands::config::save_config_file,
        commands::config::get_config_value,
        commands::config::save_config_value,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
