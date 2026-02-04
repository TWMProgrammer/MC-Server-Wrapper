use mc_server_wrapper_core::manager::ServerManager;
use tauri::State;
use std::sync::Arc;
use super::super::{CommandResult, AppError};

#[tauri::command]
pub async fn get_bedrock_versions(server_manager: State<'_, Arc<ServerManager>>) -> CommandResult<mc_server_wrapper_core::downloader::VersionManifest> {
    server_manager.get_bedrock_versions().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_velocity_versions(server_manager: State<'_, Arc<ServerManager>>) -> CommandResult<Vec<String>> {
    server_manager.get_velocity_versions().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_velocity_builds(server_manager: State<'_, Arc<ServerManager>>, version: String) -> CommandResult<Vec<String>> {
    server_manager.get_velocity_builds(&version).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_bungeecord_versions(server_manager: State<'_, Arc<ServerManager>>) -> CommandResult<Vec<String>> {
    server_manager.get_bungeecord_versions().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_minecraft_versions(server_manager: State<'_, Arc<ServerManager>>) -> CommandResult<mc_server_wrapper_core::downloader::VersionManifest> {
    server_manager.get_downloader().fetch_manifest().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_mod_loaders(server_manager: State<'_, Arc<ServerManager>>, mc_version: String, server_type: Option<String>) -> CommandResult<Vec<mc_server_wrapper_core::mod_loaders::ModLoader>> {
    server_manager.get_mod_loader_client().get_available_loaders(&mc_version, server_type.as_deref()).await.map_err(AppError::from)
}
