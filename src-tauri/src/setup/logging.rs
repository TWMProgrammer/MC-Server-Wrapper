use tauri::App;
use std::path::Path;
use anyhow::Result;

pub fn setup_logging(app: &mut App, exe_path: &Path) -> Result<()> {
    let log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    app.handle().plugin(
        tauri_plugin_log::Builder::default()
            .level(log_level)
            .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
            .targets([
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                    file_name: Some("app".to_string()),
                }),
                tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Folder {
                    path: exe_path.join("logs"),
                    file_name: Some("app".to_string()),
                }),
            ])
            .build(),
    )?;

    Ok(())
}
