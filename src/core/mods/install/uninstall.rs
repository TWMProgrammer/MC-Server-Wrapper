use std::path::Path;
use tokio::fs;
use anyhow::{Result, Context, anyhow};

/// Uninstalls a mod by removing its file and optionally its configuration folder.
pub async fn uninstall_mod(instance_path: impl AsRef<Path>, filename: String, delete_config: bool) -> Result<()> {
    // Path traversal protection
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(anyhow!("Invalid filename: {}", filename));
    }

    let mods_dir = instance_path.as_ref().join("mods");
    let mod_file = mods_dir.join(&filename);

    if mod_file.exists() {
        fs::remove_file(mod_file).await.context("Failed to delete mod file")?;
    }

    if delete_config {
        // Try to find the config directory. Mods usually put configs in the instance's 'config' directory.
        // We'll need a way to map mod IDs to config files in Phase 4.
        // For now, we'll just implement the placeholder logic.
        let instance_config_dir = instance_path.as_ref().join("config");
        if instance_config_dir.exists() {
            // TODO: In Phase 4, implement actual config discovery for mods.
        }
    }

    Ok(())
}

/// Uninstalls multiple mods at once.
pub async fn bulk_uninstall_mods(
    instance_path: impl AsRef<Path>,
    filenames: Vec<String>,
    delete_config: bool,
) -> Result<()> {
    for filename in filenames {
        uninstall_mod(&instance_path, filename, delete_config).await?;
    }
    Ok(())
}
