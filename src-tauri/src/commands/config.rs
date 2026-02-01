use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::config_files;
use mc_server_wrapper_core::app_config::{AppSettings, GlobalConfigManager};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn get_app_settings(
    config_manager: State<'_, Arc<GlobalConfigManager>>,
) -> Result<AppSettings, String> {
    config_manager.load().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_app_settings(
    config_manager: State<'_, Arc<GlobalConfigManager>>,
    settings: AppSettings,
) -> Result<(), String> {
    config_manager.save(&settings).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_config_value(
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
pub async fn save_config_value(
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
pub async fn get_available_configs(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<Vec<config_files::ConfigFile>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    Ok(config_files::list_available_configs(&instance.path, instance.mod_loader.as_deref()).await)
}

#[tauri::command]
pub async fn get_config_file(
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
pub async fn save_config_file(
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
pub async fn get_server_properties(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::read_config_file(&instance.path, "server.properties", config_files::ConfigFormat::Properties).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_server_properties(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    properties: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    config_files::save_config_file(&instance.path, "server.properties", config_files::ConfigFormat::Properties, properties).await.map_err(|e| e.to_string())
}
