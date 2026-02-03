use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::mods::{self, InstalledMod, Project, ModProvider, SearchOptions, ModConfig, ModUpdate, ResolvedDependency};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn list_installed_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> Result<Vec<InstalledMod>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::list_installed_mods(&instance.path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    enable: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::toggle_mod(&instance.path, filename, enable).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_toggle_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    enable: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::bulk_toggle_mods(&instance.path, filenames, enable).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    delete_config: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::uninstall_mod(&instance.path, filename, delete_config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_uninstall_mods(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    delete_config: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::bulk_uninstall_mods(&instance.path, filenames, delete_config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_mods(
    options: SearchOptions,
    provider: Option<ModProvider>,
) -> Result<Vec<Project>, String> {
    // Get CurseForge API key from environment if available
    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::search_mods(&options, provider, cf_api_key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_mod_dependencies(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    project_id: String,
    provider: ModProvider,
) -> Result<Vec<ResolvedDependency>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::get_mod_dependencies(
        &project_id, 
        provider, 
        Some(&instance.version), 
        instance.mod_loader.as_deref(), 
        cf_api_key
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_mod_configs(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    mod_name: String,
) -> Result<Vec<ModConfig>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::get_mod_configs(&instance.path, &mod_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_mod_config_files(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    rel_path: String,
) -> Result<Vec<String>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    mods::list_mod_config_files(&instance.path, &rel_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    project_id: String,
    provider: ModProvider,
    version_id: Option<String>,
) -> Result<String, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::install_mod(
        &instance.path, 
        &project_id, 
        provider, 
        version_id.as_deref(), 
        Some(&instance.version),
        instance.mod_loader.as_deref(),
        cf_api_key
    )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_for_mod_updates(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> Result<Vec<ModUpdate>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::check_for_updates(
        &instance.path,
        Some(&instance.version),
        instance.mod_loader.as_deref(),
        cf_api_key,
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_mod(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    project_id: String,
    provider: ModProvider,
    latest_version_id: String,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    let cf_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    mods::update_mod(
        &instance.path,
        filename,
        project_id,
        provider,
        latest_version_id,
        Some(&instance.version),
        instance.mod_loader.as_deref(),
        cf_api_key,
    ).await.map_err(|e| e.to_string())
}
