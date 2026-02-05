use mc_server_wrapper_core::java::JavaManager;
use mc_server_wrapper_core::app_config::{ManagedJavaVersion, GlobalConfigManager};
use tauri::{State, Window, Emitter};
use std::sync::Arc;
use std::path::Path;
use super::{CommandResult, AppError};

#[tauri::command]
pub async fn get_managed_java_versions(
    java_manager: State<'_, Arc<JavaManager>>,
) -> CommandResult<Vec<ManagedJavaVersion>> {
    java_manager.discover_installed_versions().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn download_java_version(
    java_manager: State<'_, Arc<JavaManager>>,
    config_manager: State<'_, Arc<GlobalConfigManager>>,
    window: Window,
    major_version: u32,
) -> CommandResult<ManagedJavaVersion> {
    let release = java_manager.get_latest_release(major_version).await
        .map_err(AppError::from)?;
    
    let release_name = release.release_name.clone();
    
    let version_info = java_manager.download_and_install(release, move |downloaded, total| {
        let _ = window.emit("java_download_progress", serde_json::json!({
            "release": release_name,
            "downloaded": downloaded,
            "total": total,
        }));
    }).await.map_err(AppError::from)?;

    // Update app settings with the new version
    let mut settings = config_manager.load().await.map_err(AppError::from)?;
    
    // Remove if already exists (update)
    settings.managed_java_versions.retain(|v| v.id != version_info.id);
    settings.managed_java_versions.push(version_info.clone());
    
    config_manager.save(&settings).await.map_err(AppError::from)?;

    Ok(version_info)
}

#[tauri::command]
pub async fn delete_java_version(
    java_manager: State<'_, Arc<JavaManager>>,
    config_manager: State<'_, Arc<GlobalConfigManager>>,
    id: String,
) -> CommandResult<()> {
    java_manager.delete_version(&id).await.map_err(AppError::from)?;

    // Update app settings
    let mut settings = config_manager.load().await.map_err(AppError::from)?;
    settings.managed_java_versions.retain(|v| v.id != id);
    config_manager.save(&settings).await.map_err(AppError::from)?;

    Ok(())
}

#[tauri::command]
pub async fn validate_custom_java(
    java_manager: State<'_, Arc<JavaManager>>,
    path: String,
) -> CommandResult<ManagedJavaVersion> {
    let path = Path::new(&path);
    
    // If it's just the executable, we need to check if its parent/parent is a JDK root
    // or just use it directly. identify_java_version expects a JDK root.
    // Let's try to find the JDK root from the executable path.
    let jdk_root = if path.ends_with("bin/java") || path.ends_with("bin/java.exe") {
        path.parent().and_then(|p| p.parent()).unwrap_or(path)
    } else {
        path
    };

    java_manager.identify_java_version(jdk_root).await
        .ok_or_else(|| AppError::Validation("Invalid Java executable or JDK directory".to_string()))
}
