use mc_server_wrapper_core::manager::ServerManager;
use tauri::State;
use std::sync::Arc;

#[tauri::command]
pub async fn get_bedrock_versions(server_manager: State<'_, Arc<ServerManager>>) -> Result<mc_server_wrapper_core::downloader::VersionManifest, String> {
    server_manager.get_bedrock_versions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_velocity_versions(server_manager: State<'_, Arc<ServerManager>>) -> Result<Vec<String>, String> {
    server_manager.get_velocity_versions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_velocity_builds(server_manager: State<'_, Arc<ServerManager>>, version: String) -> Result<Vec<String>, String> {
    server_manager.get_velocity_builds(&version).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_bungeecord_versions(server_manager: State<'_, Arc<ServerManager>>) -> Result<Vec<String>, String> {
    server_manager.get_bungeecord_versions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_minecraft_versions(server_manager: State<'_, Arc<ServerManager>>) -> Result<mc_server_wrapper_core::downloader::VersionManifest, String> {
    server_manager.get_downloader().fetch_manifest().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_mod_loaders(server_manager: State<'_, Arc<ServerManager>>, mc_version: String, server_type: Option<String>) -> Result<Vec<mc_server_wrapper_core::mod_loaders::ModLoader>, String> {
    server_manager.get_mod_loader_client().get_available_loaders(&mc_version, server_type.as_deref()).await.map_err(|e| e.to_string())
}
