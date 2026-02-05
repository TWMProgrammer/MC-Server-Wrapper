use serde::Deserialize;
use zip::ZipArchive;
use std::io::Read;
use anyhow::Result;
use crate::mods::types::InstalledMod;

#[derive(Deserialize)]
pub struct ModsToml {
    pub mods: Vec<ForgeModMetadata>,
}

#[derive(Deserialize)]
pub struct ForgeModMetadata {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<String>,
    #[serde(rename = "logoFile")]
    pub logo_file: Option<String>,
}

#[derive(Deserialize)]
pub struct LegacyForgeModInfo {
    #[serde(rename = "modList")]
    pub mod_list: Option<Vec<LegacyForgeMetadata>>,
}

#[derive(Deserialize)]
pub struct LegacyForgeMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author_list: Option<Vec<String>>,
}

pub fn parse_neoforge_toml(archive: &mut ZipArchive<std::fs::File>, mod_item: &mut InstalledMod) -> Result<Option<String>> {
    let mut icon_path = None;
    if let Ok(mut neoforge_file) = archive.by_name("META-INF/neoforge.mods.toml") {
        let mut content = String::new();
        neoforge_file.read_to_string(&mut content)?;
        if let Ok(toml_data) = toml::from_str::<ModsToml>(&content) {
            if let Some(first_mod) = toml_data.mods.first() {
                mod_item.name = first_mod.display_name.clone().unwrap_or(mod_item.name.clone());
                mod_item.version = first_mod.version.clone();
                mod_item.description = first_mod.description.clone();
                mod_item.author = first_mod.authors.clone();
                mod_item.loader = Some("NeoForge".to_string());
                icon_path = first_mod.logo_file.clone();
            }
        }
    }
    Ok(icon_path)
}

pub fn parse_forge_toml(archive: &mut ZipArchive<std::fs::File>, mod_item: &mut InstalledMod) -> Result<Option<String>> {
    let mut icon_path = None;
    if let Ok(mut forge_file) = archive.by_name("META-INF/mods.toml") {
        let mut content = String::new();
        forge_file.read_to_string(&mut content)?;
        if let Ok(toml_data) = toml::from_str::<ModsToml>(&content) {
            if let Some(first_mod) = toml_data.mods.first() {
                mod_item.name = first_mod.display_name.clone().unwrap_or(mod_item.name.clone());
                mod_item.version = first_mod.version.clone();
                mod_item.description = first_mod.description.clone();
                mod_item.author = first_mod.authors.clone();
                mod_item.loader = Some("Forge".to_string());
                icon_path = first_mod.logo_file.clone();
            }
        }
    }
    Ok(icon_path)
}

pub fn parse_legacy_forge(archive: &mut ZipArchive<std::fs::File>, mod_item: &mut InstalledMod) -> Result<()> {
    if let Ok(mut legacy_file) = archive.by_name("mcmod.info") {
        let mut content = String::new();
        legacy_file.read_to_string(&mut content)?;
        if let Ok(mods) = serde_json::from_str::<Vec<LegacyForgeMetadata>>(&content) {
            if let Some(first_mod) = mods.first() {
                mod_item.name = first_mod.name.clone().unwrap_or(mod_item.name.clone());
                mod_item.version = first_mod.version.clone();
                mod_item.description = first_mod.description.clone();
                mod_item.author = first_mod.author_list.as_ref().map(|l| l.join(", "));
                mod_item.loader = Some("Forge".to_string());
            }
        } else if let Ok(wrapped) = serde_json::from_str::<LegacyForgeModInfo>(&content) {
            if let Some(mods) = wrapped.mod_list {
                if let Some(first_mod) = mods.first() {
                    mod_item.name = first_mod.name.clone().unwrap_or(mod_item.name.clone());
                    mod_item.version = first_mod.version.clone();
                    mod_item.description = first_mod.description.clone();
                    mod_item.author = first_mod.author_list.as_ref().map(|l| l.join(", "));
                    mod_item.loader = Some("Forge".to_string());
                }
            }
        }
    }
    Ok(())
}
