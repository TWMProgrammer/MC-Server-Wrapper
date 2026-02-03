use std::path::Path;
use tokio::fs;
use anyhow::Result;
use crate::mods::types::ModConfig;

/// Lists all files in a mod's config directory/file.
pub async fn list_mod_config_files(instance_path: impl AsRef<Path>, rel_path: &str) -> Result<Vec<String>> {
    let full_path = instance_path.as_ref().join(rel_path);
    if !full_path.exists() {
        return Ok(vec![]);
    }

    if full_path.is_file() {
        if let Ok(rel) = full_path.strip_prefix(instance_path.as_ref().join("config")) {
            return Ok(vec![rel.to_string_lossy().to_string().replace('\\', "/")]);
        }
        return Ok(vec![rel_path.to_string()]);
    }

    let mut files = Vec::new();
    let mut stack = vec![full_path.clone()];

    while let Some(current_dir) = stack.pop() {
        let mut entries = fs::read_dir(&current_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                // Get path relative to the config directory root
                if let Ok(rel) = path.strip_prefix(instance_path.as_ref().join("config")) {
                    files.push(rel.to_string_lossy().to_string().replace('\\', "/"));
                }
            }
        }
    }

    Ok(files)
}

pub async fn get_mod_configs(instance_path: impl AsRef<Path>, mod_name: &str) -> Result<Vec<ModConfig>> {
    let config_dir = instance_path.as_ref().join("config");
    if !config_dir.exists() {
        return Ok(vec![]);
    }

    let mut configs = Vec::new();
    let mut entries = fs::read_dir(&config_dir).await?;

    // Try to find files/dirs that match the mod name or a common ID pattern
    // e.g., for "Fabric API", look for "fabric-api.json" or "fabric"
    let mod_id = mod_name.to_lowercase().replace(' ', "-");
    let mod_id_short = mod_id.split('-').next().unwrap_or(&mod_id);

    while let Some(entry) = entries.next_entry().await? {
        let file_name = entry.file_name().to_string_lossy().to_lowercase();
        let is_match = file_name.contains(&mod_id) || 
                      file_name.contains(mod_id_short) ||
                      mod_id.contains(&file_name);

        if is_match {
            let file_type = entry.file_type().await?;
            configs.push(ModConfig {
                name: entry.file_name().to_string_lossy().to_string(),
                path: format!("config/{}", entry.file_name().to_string_lossy()),
                is_dir: file_type.is_dir(),
            });
        }
    }

    Ok(configs)
}
