use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::config_files;
use mc_server_wrapper_core::app_config::{AppSettings, GlobalConfigManager};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;
use super::{CommandResult, AppError};

#[tauri::command]
pub async fn get_app_settings(
    config_manager: State<'_, Arc<GlobalConfigManager>>,
) -> CommandResult<AppSettings> {
    config_manager.load().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn update_app_settings(
    config_manager: State<'_, Arc<GlobalConfigManager>>,
    settings: AppSettings,
) -> CommandResult<()> {
    config_manager.save(&settings).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_config_value(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
) -> CommandResult<serde_json::Value> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    config_files::read_config_value(&instance.path, &rel_path, format).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn save_config_value(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
    value: serde_json::Value,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    config_files::save_config_value(&instance.path, &rel_path, format, value).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_available_configs(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> CommandResult<Vec<config_files::ConfigFile>> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    Ok(config_files::list_available_configs(&instance.path, instance.mod_loader.as_deref()).await)
}

#[tauri::command]
pub async fn get_config_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
) -> CommandResult<std::collections::HashMap<String, String>> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    config_files::read_config_file(&instance.path, &rel_path, format).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn save_config_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    rel_path: String,
    format: config_files::ConfigFormat,
    properties: std::collections::HashMap<String, String>,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    config_files::save_config_file(&instance.path, &rel_path, format, properties).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_server_properties(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> CommandResult<std::collections::HashMap<String, String>> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    config_files::read_config_file(&instance.path, "server.properties", config_files::ConfigFormat::Properties).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn save_server_properties(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    properties: std::collections::HashMap<String, String>,
) -> CommandResult<()> {
    let id = Uuid::parse_str(&instance_id).map_err(AppError::from)?;
    let instance = instance_manager.get_instance(id).await.map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Instance not found".to_string()))?;
    
    // Save the properties file
    config_files::save_config_file(&instance.path, "server.properties", config_files::ConfigFormat::Properties, properties.clone()).await.map_err(AppError::from)?;

    // If the port changed, update the instance settings in the DB
    if let Some(port_str) = properties.get("server-port") {
        if let Ok(port) = port_str.parse::<u16>() {
            if port != instance.settings.port {
                let mut new_settings = instance.settings.clone();
                new_settings.port = port;
                instance_manager.update_settings(id, None, new_settings).await.map_err(AppError::from)?;
            }
        }
    }

    Ok(())
}
