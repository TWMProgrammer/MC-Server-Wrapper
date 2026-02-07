use crate::commands::CommandResult;
use log::{error, info};
use mc_server_wrapper_core::database::explorer::{self, ColumnInfo, DatabaseGroup, TableData};
use mc_server_wrapper_core::errors::AppError;
use mc_server_wrapper_core::instance::InstanceManager;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn explore_find_databases(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> CommandResult<Vec<DatabaseGroup>> {
    info!("Exploring databases for instance: {}", instance_id);
    let id = Uuid::parse_str(&instance_id).map_err(|e| {
        error!("Failed to parse instance UUID: {}", e);
        AppError::from(e)
    })?;

    let instance = instance_manager
        .get_instance(id)
        .await
        .map_err(|e| {
            error!("Failed to get instance from DB: {}", e);
            AppError::from(e)
        })?
        .ok_or_else(|| {
            error!("Instance not found: {}", id);
            AppError::NotFound(format!("Instance not found: {}", instance_id))
        })?;

    info!("Scanning for databases in: {:?}", instance.path);
    let dbs = explorer::find_database_files(&instance.path);
    info!("Found {} database groups", dbs.len());
    Ok(dbs)
}

#[tauri::command]
pub async fn explore_list_tables(path: PathBuf) -> CommandResult<Vec<String>> {
    explorer::list_tables(&path).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn explore_get_data(
    path: PathBuf,
    table: String,
    limit: u32,
    offset: u32,
) -> CommandResult<TableData> {
    explorer::get_table_data(&path, &table, limit, offset)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn explore_read_sql_file(path: PathBuf) -> CommandResult<String> {
    explorer::read_sql_file(&path).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn explore_get_schema(path: PathBuf, table: String) -> CommandResult<Vec<ColumnInfo>> {
    explorer::get_table_columns(&path, &table)
        .await
        .map_err(|e| e.into())
}
