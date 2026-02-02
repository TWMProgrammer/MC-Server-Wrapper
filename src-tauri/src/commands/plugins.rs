use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::plugins::{self, InstalledPlugin, Project, PluginProvider, SearchOptions, PluginUpdate};
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn list_installed_plugins(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> Result<Vec<InstalledPlugin>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::list_installed_plugins(&instance.path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    enable: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::toggle_plugin(&instance.path, filename, enable).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_toggle_plugins(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    enable: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::bulk_toggle_plugins(&instance.path, filenames, enable).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn uninstall_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    delete_config: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::uninstall_plugin(&instance.path, filename, delete_config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_uninstall_plugins(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filenames: Vec<String>,
    delete_config: bool,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::bulk_uninstall_plugins(&instance.path, filenames, delete_config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_for_plugin_updates(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
) -> Result<Vec<PluginUpdate>, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::check_for_updates(&instance.path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    filename: String,
    project_id: String,
    provider: PluginProvider,
    latest_version_id: String,
) -> Result<(), String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::update_plugin(&instance.path, filename, project_id, provider, latest_version_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_plugins(
    options: SearchOptions,
    provider: Option<PluginProvider>,
) -> Result<Vec<Project>, String> {
    plugins::search_plugins(&options, provider).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_plugin_dependencies(
    project_id: String,
    provider: PluginProvider,
) -> Result<Vec<Project>, String> {
    plugins::get_plugin_dependencies(&project_id, provider).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_plugin(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    project_id: String,
    provider: PluginProvider,
    version_id: Option<String>,
) -> Result<String, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    plugins::install_plugin(
        &instance.path, 
        &project_id, 
        provider, 
        version_id.as_deref(),
        Some(&instance.version),
        instance.mod_loader.as_deref()
    )
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct PluginConfigs {
    pub config_dir: String,
    pub files: Vec<String>,
}

#[tauri::command]
pub async fn list_plugin_configs(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: Uuid,
    plugin_name: String,
    plugin_filename: String,
) -> Result<PluginConfigs, String> {
    let instances = instance_manager.list_instances().await.map_err(|e| e.to_string())?;
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

    let plugins_dir = instance.path.join("plugins");
    
    // Attempt to find the config directory by checking multiple possible names
    
    let base_name_from_file = if plugin_filename.ends_with(".jar.disabled") {
        plugin_filename.strip_suffix(".jar.disabled").unwrap()
    } else {
        plugin_filename.strip_suffix(".jar").unwrap_or(&plugin_filename)
    };

    // Try multiple possible directory names
    let mut possible_dirs = vec![
        plugin_name.clone(),
        plugin_name.to_lowercase(),
        plugin_name.replace(' ', "_"),
        plugin_name.replace(' ', ""),
        base_name_from_file.to_string(),
        base_name_from_file.to_lowercase(),
        base_name_from_file.replace(' ', "_"),
        base_name_from_file.replace(' ', ""),
    ];

    // Add a more aggressive version stripper for the filename
    // e.g., "TAB v5.5.0" -> "TAB"
    if let Some(stripped) = base_name_from_file.split_once(' ') {
        possible_dirs.push(stripped.0.to_string());
    }
    if let Some(stripped) = base_name_from_file.split_once('-') {
        possible_dirs.push(stripped.0.to_string());
    }
    if let Some(stripped) = base_name_from_file.split_once('_') {
        possible_dirs.push(stripped.0.to_string());
    }

    // Remove duplicates and keep order
    let mut seen = std::collections::HashSet::new();
    possible_dirs.retain(|x| seen.insert(x.clone()));

    let mut found_config_dir = None;
    let mut found_dir_name = String::new();

    for dir_name in possible_dirs {
        let path = plugins_dir.join(&dir_name);
        if path.exists() && path.is_dir() {
            found_config_dir = Some(path);
            found_dir_name = dir_name;
            break;
        }
    }

    let config_dir = match found_config_dir {
        Some(dir) => dir,
        None => return Ok(PluginConfigs { config_dir: String::new(), files: vec![] }),
    };

    let mut files = Vec::new();
    let mut stack = vec![(config_dir.clone(), String::new())];
    
    while let Some((dir, prefix)) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&dir).await.map_err(|e| e.to_string())?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let rel_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            if path.is_dir() {
                stack.push((path, rel_path));
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ["yml", "yaml", "json", "txt", "properties", "conf", "log", "sk", "csv", "toml", "xml", "lua", "js", "css", "html", "md"].contains(&ext.as_str()) {
                    files.push(rel_path);
                }
            }
        }
    }
    
    // Sort alphabetically
    files.sort();
    
    Ok(PluginConfigs {
        config_dir: found_dir_name,
        files,
    })
}
