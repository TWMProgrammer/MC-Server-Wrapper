pub mod fabric;
pub mod forge;
pub mod quilt;

use zip::ZipArchive;
use anyhow::Result;
use crate::mods::types::InstalledMod;
use base64::{Engine as _, engine::general_purpose};
use std::io::Read;

pub fn extract_metadata_sync(path: &std::path::Path) -> Result<InstalledMod> {
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    let is_disabled = filename.ends_with(".disabled");
    
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut mod_item = InstalledMod {
        name: filename.clone(),
        filename: filename.clone(),
        enabled: !is_disabled,
        version: None,
        author: None,
        description: None,
        loader: None,
        source: None,
        icon_data: None,
    };

    // Try Fabric
    let mut icon_path = fabric::parse_fabric(&mut archive, &mut mod_item)?;

    // Try NeoForge
    if mod_item.loader.is_none() {
        icon_path = forge::parse_neoforge_toml(&mut archive, &mut mod_item)?;
    }

    if let Some(ip) = icon_path {
        if let Ok(mut icon_file) = archive.by_name(&ip) {
            let mut buffer = Vec::new();
            icon_file.read_to_end(&mut buffer)?;
            mod_item.icon_data = Some(general_purpose::STANDARD.encode(buffer));
        }
    }

    if mod_item.loader.is_none() {
        // Try Quilt
        let quilt_icon_path = quilt::parse_quilt(&mut archive, &mut mod_item)?;
        if let Some(ip) = quilt_icon_path {
            if let Ok(mut icon_file) = archive.by_name(&ip) {
                let mut buffer = Vec::new();
                icon_file.read_to_end(&mut buffer)?;
                mod_item.icon_data = Some(general_purpose::STANDARD.encode(buffer));
            }
        }
    }

    if mod_item.loader.is_none() {
        // Try Forge/NeoForge (mods.toml)
        let forge_icon_path = forge::parse_forge_toml(&mut archive, &mut mod_item)?;
        if let Some(logo) = forge_icon_path {
            if let Ok(mut icon_file) = archive.by_name(&logo) {
                let mut buffer = Vec::new();
                icon_file.read_to_end(&mut buffer)?;
                mod_item.icon_data = Some(general_purpose::STANDARD.encode(buffer));
            }
        }
    }

    if mod_item.loader.is_none() {
        // Try Legacy Forge (mcmod.info)
        forge::parse_legacy_forge(&mut archive, &mut mod_item)?;
    }

    // Fallback name cleaning if still using filename
    if mod_item.name == filename {
        let name = if is_disabled {
            filename.strip_suffix(".jar.disabled").unwrap_or(&filename).to_string()
        } else {
            filename.strip_suffix(".jar").unwrap_or(&filename).to_string()
        };
        mod_item.name = name;
    }

    Ok(mod_item)
}
