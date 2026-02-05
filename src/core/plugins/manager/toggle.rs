use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context};

/// Toggles a plugin's enabled state by renaming the file.
pub async fn toggle_plugin(instance_path: impl AsRef<Path>, filename: String, enable: bool) -> Result<()> {
    let plugins_dir = instance_path.as_ref().join("plugins");
    let current_path = plugins_dir.join(&filename);
    
    if !current_path.exists() {
        return Err(anyhow::anyhow!("Plugin file not found: {}", filename));
    }

    let new_filename = if enable {
        if !filename.ends_with(".jar.disabled") {
            return Ok(()); // Already enabled or not a disabled plugin
        }
        filename.strip_suffix(".disabled").unwrap().to_string()
    } else {
        if filename.ends_with(".jar.disabled") {
            return Ok(()); // Already disabled
        }
        format!("{}.disabled", filename)
    };

    let new_path = plugins_dir.join(new_filename);
    fs::rename(current_path, new_path).await.context("Failed to rename plugin file")?;

    Ok(())
}

/// Toggles multiple plugins at once.
pub async fn bulk_toggle_plugins(
    instance_path: impl AsRef<Path>,
    filenames: Vec<String>,
    enable: bool,
) -> Result<()> {
    for filename in filenames {
        let _ = toggle_plugin(&instance_path, filename, enable).await;
    }
    Ok(())
}
