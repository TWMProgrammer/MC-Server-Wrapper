use serde::Deserialize;
use zip::ZipArchive;
use std::io::Read;
use anyhow::Result;
use std::collections::HashMap;
use crate::mods::types::InstalledMod;

#[derive(Deserialize)]
pub struct QuiltModJson {
    pub quilt_loader: QuiltLoaderMetadata,
}

#[derive(Deserialize)]
pub struct QuiltLoaderMetadata {
    pub metadata: Option<QuiltMetadata>,
}

#[derive(Deserialize)]
pub struct QuiltMetadata {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub contributors: Option<HashMap<String, String>>,
    pub icon: Option<serde_json::Value>,
}

pub fn parse_quilt(archive: &mut ZipArchive<std::fs::File>, mod_item: &mut InstalledMod) -> Result<Option<String>> {
    let mut icon_path = None;
    if let Ok(mut quilt_file) = archive.by_name("quilt.mod.json") {
        let mut content = String::new();
        quilt_file.read_to_string(&mut content)?;
        if let Ok(json) = serde_json::from_str::<QuiltModJson>(&content) {
            if let Some(metadata) = json.quilt_loader.metadata {
                mod_item.name = metadata.name.unwrap_or(mod_item.name.clone());
                mod_item.version = metadata.version;
                mod_item.description = metadata.description;
                mod_item.loader = Some("Quilt".to_string());

                if let Some(contributors) = metadata.contributors {
                    let author_names: Vec<String> = contributors.keys().cloned().collect();
                    mod_item.author = Some(author_names.join(", "));
                }

                if let Some(icon) = metadata.icon {
                    icon_path = if icon.is_string() {
                        icon.as_str().map(|s| s.to_string())
                    } else if icon.is_object() {
                        icon.get("128").or_else(|| icon.get("64")).or_else(|| icon.get("32"))
                            .and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    };
                }
            }
        }
    }
    Ok(icon_path)
}
