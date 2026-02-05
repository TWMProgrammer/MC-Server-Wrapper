use serde::Deserialize;
use zip::ZipArchive;
use std::io::Read;
use anyhow::Result;
use crate::mods::types::InstalledMod;

#[derive(Deserialize)]
pub struct FabricModJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub authors: Option<serde_json::Value>,
    pub icon: Option<serde_json::Value>,
}

pub fn parse_fabric(archive: &mut ZipArchive<std::fs::File>, mod_item: &mut InstalledMod) -> Result<Option<String>> {
    let mut icon_path = None;
    if let Ok(mut fabric_file) = archive.by_name("fabric.mod.json") {
        let mut content = String::new();
        fabric_file.read_to_string(&mut content)?;
        if let Ok(json) = serde_json::from_str::<FabricModJson>(&content) {
            mod_item.name = json.name.unwrap_or(mod_item.name.clone());
            mod_item.version = json.version;
            mod_item.description = json.description;
            mod_item.loader = Some("Fabric".to_string());
            
            if let Some(authors) = json.authors {
                if authors.is_array() {
                    let author_names: Vec<String> = authors.as_array().unwrap().iter()
                        .filter_map(|a| {
                            if a.is_string() {
                                Some(a.as_str().unwrap().to_string())
                            } else if a.is_object() {
                                a.get("name").and_then(|n| n.as_str()).map(|n| n.to_string())
                            } else {
                                None
                            }
                        }).collect();
                    mod_item.author = Some(author_names.join(", "));
                } else if authors.is_string() {
                    mod_item.author = Some(authors.as_str().unwrap().to_string());
                }
            }

            if let Some(icon) = json.icon {
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
    Ok(icon_path)
}
