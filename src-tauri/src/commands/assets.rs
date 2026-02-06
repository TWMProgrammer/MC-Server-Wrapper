use crate::commands::CommandResult;
use mc_server_wrapper_core::assets::{AssetCacheStats, AssetManager};
use mc_server_wrapper_core::errors::AppError;
use std::sync::Arc;
use std::time::Duration;
use tauri::State;

#[tauri::command]
pub async fn cache_asset(
    asset_manager: State<'_, Arc<AssetManager>>,
    url: String,
) -> CommandResult<String> {
    let path = asset_manager
        .get_asset(&url)
        .await
        .map_err(AppError::from)?;

    // Return the path as a string.
    // In Tauri, we can use the `convertFileSrc` on the frontend or the `asset:` protocol.
    // For simplicity, we return the absolute path.
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_player_head_path(
    asset_manager: State<'_, Arc<AssetManager>>,
    uuid: String,
) -> CommandResult<String> {
    let path = asset_manager
        .get_player_head(&uuid)
        .await
        .map_err(AppError::from)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_asset_cache_stats(
    asset_manager: State<'_, Arc<AssetManager>>,
) -> CommandResult<AssetCacheStats> {
    let stats = asset_manager.get_stats().await.map_err(AppError::from)?;
    Ok(stats)
}

#[tauri::command]
pub async fn cleanup_assets(
    asset_manager: State<'_, Arc<AssetManager>>,
    max_age_days: Option<u64>,
) -> CommandResult<u64> {
    let max_age = Duration::from_secs(max_age_days.unwrap_or(7) * 24 * 60 * 60);
    let count = asset_manager
        .cleanup_assets(max_age)
        .await
        .map_err(AppError::from)?;
    Ok(count)
}
