use super::{AppError, CommandResult};
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::mods::{
    self, InstalledMod, ModConfig, ModProvider, ModUpdate, Project, ResolvedDependency,
    SearchOptions,
};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn list_installed_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> CommandResult<Vec<InstalledMod>> {
    let instances = instance_manager
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    mods::list_installed_mods(&instance.path)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn toggle_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    enable: bool,
) -> CommandResult<()> {
    let instances = instance_manager
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    mods::toggle_mod(&instance.path, filename, enable)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn bulk_toggle_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    enable: bool,
) -> CommandResult<()> {
    let instances = instance_manager
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    mods::bulk_toggle_mods(&instance.path, filenames, enable)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn uninstall_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    delete_config: bool,
) -> CommandResult<()> {
    let instances = instance_manager
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    mods::uninstall_mod(&instance.path, filename, delete_config)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn bulk_uninstall_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    delete_config: bool,
) -> CommandResult<()> {
    let instances = instance_manager
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    mods::bulk_uninstall_mods(&instance.path, filenames, delete_config)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn search_mods(
    server_manager: State<'_, Arc<ServerManager>>,
    options: SearchOptions,
    provider: Option<ModProvider>,
) -> CommandResult<Vec<Project>> {
    // Get CurseForge API key from environment if available
    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::search_mods(&options, provider, cf_api_key, server_manager.get_cache())
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn get_mod_dependencies(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: Uuid,
    project_id: String,
    provider: ModProvider,
) -> CommandResult<Vec<ResolvedDependency>> {
    let instances = server_manager
        .get_instance_manager()
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::get_mod_dependencies(
        &project_id,
        provider,
        Some(instance.version.as_str()),
        instance.mod_loader.as_deref(),
        cf_api_key,
        server_manager.get_cache(),
    )
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn get_mod_versions(
    server_manager: State<'_, Arc<ServerManager>>,
    project_id: String,
    provider: ModProvider,
    game_version: Option<String>,
    loader: Option<String>,
) -> CommandResult<Vec<mc_server_wrapper_core::mods::ProjectVersion>> {
    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();

    match provider {
        ModProvider::Modrinth => {
            let client =
                mc_server_wrapper_core::mods::ModrinthClient::new(server_manager.get_cache());
            client
                .get_versions(&project_id, game_version.as_deref(), loader.as_deref())
                .await
                .map_err(AppError::from)
        }
        ModProvider::CurseForge => {
            let client = mc_server_wrapper_core::mods::CurseForgeClient::new(
                cf_api_key,
                server_manager.get_cache(),
            );
            client
                .get_versions(&project_id, game_version.as_deref(), loader.as_deref())
                .await
                .map_err(AppError::from)
        }
    }
}

#[tauri::command]
pub async fn get_mod_configs(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    mod_name: String,
) -> CommandResult<Vec<ModConfig>> {
    let instances = instance_manager
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    mods::get_mod_configs(&instance.path, &mod_name)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn list_mod_config_files(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    rel_path: String,
) -> CommandResult<Vec<String>> {
    let instances = instance_manager
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    mods::list_mod_config_files(&instance.path, &rel_path)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn install_mod(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: Uuid,
    project_id: String,
    provider: ModProvider,
    version_id: Option<String>,
) -> CommandResult<()> {
    let instances = server_manager
        .get_instance_manager()
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::install_mod(
        &instance.path,
        &project_id,
        provider,
        version_id.as_deref(),
        Some(instance.version.as_str()),
        instance.mod_loader.as_deref(),
        cf_api_key,
        server_manager.get_cache(),
    )
    .await
    .map(|_| ())
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn check_for_mod_updates(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: Uuid,
) -> CommandResult<Vec<ModUpdate>> {
    let instances = server_manager
        .get_instance_manager()
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::check_for_updates(
        &instance.path,
        Some(instance.version.as_str()),
        instance.mod_loader.as_deref(),
        cf_api_key,
        server_manager.get_cache(),
    )
    .await
    .map_err(AppError::from)
}

#[tauri::command]
pub async fn update_mod(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: Uuid,
    updates: Vec<ModUpdate>,
) -> CommandResult<()> {
    let instances = server_manager
        .get_instance_manager()
        .list_instances()
        .await
        .map_err(AppError::from)?;
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance not found: {}", instance_id)))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();

    for update in updates {
        mods::update_mod(
            &instance.path,
            update.filename,
            update.project_id,
            update.provider,
            update.latest_version_id,
            Some(instance.version.as_str()),
            instance.mod_loader.as_deref(),
            cf_api_key.clone(),
            server_manager.get_cache(),
        )
        .await
        .map_err(AppError::from)?;
    }

    Ok(())
}
