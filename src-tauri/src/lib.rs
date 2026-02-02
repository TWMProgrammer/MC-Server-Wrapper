mod commands;

use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::backup::BackupManager;
use mc_server_wrapper_core::scheduler::SchedulerManager;
use mc_server_wrapper_core::java_manager::JavaManager;
use mc_server_wrapper_core::app_config::{GlobalConfigManager, CloseBehavior};
use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
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

      // System Tray
      let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

      let _tray = TrayIconBuilder::new()
          .icon(app.default_window_icon().unwrap().clone())
          .menu(&menu)
          .on_menu_event(|app, event| {
              match event.id.as_ref() {
                  "quit" => {
                      app.exit(0);
                  }
                  "show" => {
                      if let Some(window) = app.get_webview_window("main") {
                          let _ = window.show();
                          let _ = window.set_focus();
                      }
                  }
                  _ => {}
              }
          })
          .on_tray_icon_event(|tray, event| {
              if let TrayIconEvent::Click { .. } = event {
                  let app = tray.app_handle();
                  if let Some(window) = app.get_webview_window("main") {
                      let _ = window.show();
                      let _ = window.set_focus();
                  }
              }
          })
          .build(app)?;

      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      app.handle().plugin(tauri_plugin_dialog::init())?;
      app.handle().plugin(tauri_plugin_opener::init())?;

      // Initialize Directories next to the executable
      let exe_path = std::env::current_exe()
          .map(|p| p.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| p))
          .expect("failed to get exe directory");
      let app_dirs = tauri::async_runtime::block_on(async {
          mc_server_wrapper_core::init::init_directories(&exe_path).await.expect("failed to initialize directories")
      });
      
      // Initialize GlobalConfigManager
      let config_manager = Arc::new(GlobalConfigManager::new(exe_path.join("app_settings.json")));
      
      // Initialize JavaManager
      let java_manager = Arc::new(JavaManager::new().expect("failed to initialize java manager"));

      // Initialize InstanceManager using the 'server' directory
      let instance_manager = Arc::new(tauri::async_runtime::block_on(async {
          InstanceManager::new(app_dirs.server).await.expect("failed to initialize instance manager")
      }));

      let server_manager = Arc::new(ServerManager::new(Arc::clone(&instance_manager), Arc::clone(&config_manager)));
      let backup_manager = Arc::new(BackupManager::new(app_dirs.backups));
      let scheduler_manager = Arc::new(tauri::async_runtime::block_on(async {
          let sm = SchedulerManager::new(Arc::clone(&server_manager), Arc::clone(&backup_manager)).await.expect("failed to initialize scheduler manager");
          
          // Load existing schedules
          let instances = instance_manager.list_instances().await.unwrap_or_default();
          for instance in instances {
              for task in instance.schedules {
                  if task.enabled {
                      let _ = sm.add_task(task).await;
                  }
              }
          }
          sm
      }));

      app.manage(instance_manager);
      app.manage(server_manager);
      app.manage(backup_manager);
      app.manage(scheduler_manager);
      app.manage(config_manager);
      app.manage(java_manager);
      app.manage(AppState {
          subscribed_servers: Arc::new(TokioMutex::new(HashSet::new())),
      });
      
      Ok(())
    })
    .on_window_event(|window, event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            let app_handle = window.app_handle();
            let config_manager = app_handle.state::<Arc<GlobalConfigManager>>();
            
            // We need to block on this because on_window_event is sync
            let settings = tauri::async_runtime::block_on(async {
                config_manager.load().await.unwrap_or_default()
            });

            match settings.close_behavior {
                CloseBehavior::HideToSystemTray => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                CloseBehavior::HideToTaskbar => {
                    api.prevent_close();
                    let _ = window.minimize();
                }
                CloseBehavior::Exit => {
                    // Let the window close
                }
            }
        }
    })
    .invoke_handler(tauri::generate_handler![
        commands::config::get_app_settings,
        commands::config::update_app_settings,
        commands::files::read_text_file,
        commands::files::save_text_file,
        commands::files::open_file_in_editor,
        commands::instance::list_instances,
        commands::instance::create_instance,
        commands::instance::import_instance,
        commands::instance::list_archive_contents,
        commands::instance::detect_server_type,
        commands::instance::list_jars_in_source,
        commands::instance::check_server_properties_exists,
        commands::instance::delete_instance,
        commands::instance::clone_instance,
        commands::instance::open_instance_folder,
        commands::instance::get_minecraft_versions,
        commands::instance::get_bedrock_versions,
        commands::instance::get_velocity_versions,
        commands::instance::get_velocity_builds,
        commands::instance::get_bungeecord_versions,
        commands::instance::get_mod_loaders,
        commands::instance::create_instance_full,
        commands::instance::update_instance_settings,
        commands::instance::update_instance_jar,
        commands::instance::get_startup_preview,
        commands::instance::list_bat_files,
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
        commands::backups::list_backups,
        commands::backups::create_backup,
        commands::backups::delete_backup,
        commands::backups::restore_backup,
        commands::backups::open_backup,
        commands::scheduler::add_scheduled_task,
        commands::scheduler::remove_scheduled_task,
        commands::scheduler::list_scheduled_tasks,
        commands::java::get_managed_java_versions,
        commands::java::download_java_version,
        commands::java::delete_java_version,
        commands::java::validate_custom_java,
        commands::plugins::list_installed_plugins,
        commands::plugins::toggle_plugin,
        commands::plugins::bulk_toggle_plugins,
        commands::plugins::uninstall_plugin,
        commands::plugins::bulk_uninstall_plugins,
        commands::plugins::search_plugins,
        commands::plugins::install_plugin,
        commands::plugins::update_plugin,
        commands::plugins::check_for_plugin_updates,
        commands::plugins::list_plugin_configs,
        commands::plugins::get_plugin_dependencies,
        commands::mods::list_installed_mods,
        commands::mods::toggle_mod,
        commands::mods::bulk_toggle_mods,
        commands::mods::uninstall_mod,
        commands::mods::bulk_uninstall_mods,
        commands::mods::search_mods,
        commands::mods::install_mod,
        commands::mods::get_mod_dependencies,
        commands::mods::get_mod_configs,
        commands::mods::list_mod_config_files,
        commands::mods::check_for_mod_updates,
        commands::mods::update_mod,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
