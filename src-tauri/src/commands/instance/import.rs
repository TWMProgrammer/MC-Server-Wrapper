use mc_server_wrapper_core::instance::{InstanceManager, InstanceMetadata};
use tauri::{State, Emitter};
use std::sync::Arc;
use std::path::PathBuf;
use std::collections::HashSet;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ZipEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImportProgressPayload {
    pub current: u64,
    pub total: u64,
    pub message: String,
}

#[tauri::command]
pub async fn import_instance(
    app_handle: tauri::AppHandle,
    instance_manager: State<'_, Arc<InstanceManager>>,
    name: String,
    source_path: String,
    jar_name: String,
    server_type: String,
    root_within_zip: Option<String>,
) -> Result<InstanceMetadata, String> {
    let path = PathBuf::from(source_path);
    let mod_loader = if server_type == "vanilla" || server_type == "custom" {
        None
    } else {
        Some(server_type)
    };
    
    let app_handle_clone = app_handle.clone();
    instance_manager.import_instance(&name, path, jar_name, mod_loader, root_within_zip, move |current, total, message| {
        let _ = app_handle_clone.emit("import-progress", ImportProgressPayload {
            current,
            total,
            message,
        });
    })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_jars_in_source(source_path: String, root_within_zip: Option<String>) -> Result<Vec<String>, String> {
    let path = PathBuf::from(&source_path);
    let mut jars = Vec::new();

    if path.is_dir() {
        let mut entries = tokio::fs::read_dir(&path).await.map_err(|e| e.to_string())?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(extension) = entry_path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "jar" {
                        if let Some(file_name) = entry_path.file_name() {
                            jars.push(file_name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    } else if path.is_file() {
        let extension = path.extension().map_or("", |ext| ext.to_str().unwrap_or("")).to_lowercase();
        if extension == "zip" {
            let file = std::fs::File::open(&path).map_err(|e: std::io::Error| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e: zip::result::ZipError| e.to_string())?;
            
            let root = root_within_zip.map(|r| {
                if r.ends_with('/') { r } else { format!("{}/", r) }
            });

            for i in 0..archive.len() {
                let file = archive.by_index(i).map_err(|e: zip::result::ZipError| e.to_string())?;
                let name = file.name();
                
                if let Some(ref root_path) = root {
                    if !name.starts_with(root_path) {
                        continue;
                    }
                    let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                    if !file.is_dir() && relative_name.to_lowercase().ends_with(".jar") && !relative_name.contains('/') {
                        jars.push(relative_name.to_string());
                    }
                } else if !file.is_dir() && name.to_lowercase().ends_with(".jar") {
                    jars.push(name.to_string());
                }
            }
        } else if extension == "7z" {
            let root = root_within_zip.as_deref().map(|r| {
                if r.ends_with('/') { r.to_string() } else { format!("{}/", r) }
            });

            sevenz_rust::SevenZReader::open(&path, "".into()).map_err(|e| e.to_string())?.for_each_entries(|entry, _| {
                let name = entry.name();
                if let Some(ref root_path) = root {
                    if !name.starts_with(root_path) {
                        return Ok(true);
                    }
                    let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                    if !entry.is_directory() && relative_name.to_lowercase().ends_with(".jar") && !relative_name.contains('/') {
                        jars.push(relative_name.to_string());
                    }
                } else if !entry.is_directory() && name.to_lowercase().ends_with(".jar") {
                    jars.push(name.to_string());
                }
                Ok(true)
            }).map_err(|e| e.to_string())?;
        }
    }

    Ok(jars)
}

#[tauri::command]
pub async fn check_server_properties_exists(source_path: String, root_within_zip: Option<String>) -> Result<bool, String> {
    let path = PathBuf::from(&source_path);

    if path.is_dir() {
        Ok(path.join("server.properties").exists())
    } else if path.is_file() {
        let extension = path.extension().map_or("", |ext| ext.to_str().unwrap_or("")).to_lowercase();
        if extension == "zip" {
            let file = std::fs::File::open(&path).map_err(|e: std::io::Error| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e: zip::result::ZipError| e.to_string())?;
            
            let target = if let Some(root) = root_within_zip {
                let root = if root.ends_with('/') { root } else { format!("{}/", root) };
                format!("{}server.properties", root)
            } else {
                "server.properties".to_string()
            };

            for i in 0..archive.len() {
                let file = archive.by_index(i).map_err(|e: zip::result::ZipError| e.to_string())?;
                if file.name() == target {
                    return Ok(true);
                }
            }
            Ok(false)
        } else if extension == "7z" {
            let target = if let Some(root) = root_within_zip {
                let root = if root.ends_with('/') { root } else { format!("{}/", root) };
                format!("{}server.properties", root)
            } else {
                "server.properties".to_string()
            };

            let mut exists = false;
            sevenz_rust::SevenZReader::open(&path, "".into()).map_err(|e| e.to_string())?.for_each_entries(|entry, _| {
                if entry.name() == target {
                    exists = true;
                    return Ok(false); // Stop iterating
                }
                Ok(true)
            }).map_err(|e| e.to_string())?;
            Ok(exists)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn detect_server_type(source_path: String, root_within_zip: Option<String>) -> Result<String, String> {
    let path = PathBuf::from(&source_path);
    let mut files = HashSet::new();
    let mut folders = HashSet::new();

    if path.is_dir() {
        let mut entries = tokio::fs::read_dir(&path).await.map_err(|e| e.to_string())?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            let entry_path = entry.path();
            if let Some(name) = entry_path.file_name().map(|n| n.to_string_lossy().to_string()) {
                if entry_path.is_dir() {
                    folders.insert(name);
                } else {
                    files.insert(name);
                }
            }
        }
    } else if path.is_file() {
        let extension = path.extension().map_or("", |ext| ext.to_str().unwrap_or("")).to_lowercase();
        if extension == "zip" {
            let file = std::fs::File::open(&path).map_err(|e: std::io::Error| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e: zip::result::ZipError| e.to_string())?;
            
            let root = root_within_zip.map(|r| {
                if r.ends_with('/') { r } else { format!("{}/", r) }
            });

            for i in 0..archive.len() {
                let file = archive.by_index(i).map_err(|e: zip::result::ZipError| e.to_string())?;
                let entry_name = file.name().to_string();

                if let Some(ref root_path) = root {
                    if !entry_name.starts_with(root_path) {
                        continue;
                    }
                    let relative_name = entry_name.strip_prefix(root_path).unwrap_or(&entry_name);
                    if relative_name.is_empty() {
                        continue;
                    }

                    if let Some(first_part) = relative_name.split('/').next() {
                        if relative_name.ends_with('/') && !relative_name.trim_end_matches('/').contains('/') {
                            folders.insert(first_part.to_string());
                        } else if !relative_name.contains('/') {
                            files.insert(first_part.to_string());
                        }
                    }

                    if relative_name.starts_with("libraries/") {
                        folders.insert("libraries".to_string());
                        if relative_name.contains("net/minecraftforge") {
                            files.insert("forge_marker".to_string());
                        }
                        if relative_name.contains("net/fabricmc") {
                            files.insert("fabric_marker".to_string());
                        }
                    }
                } else {
                    // Handle both files and directories in ZIP. ZipArchive::by_index returns full path.
                    // We only care about top-level files/folders for basic detection.
                    if let Some(first_part) = entry_name.split('/').next() {
                        if entry_name.ends_with('/') {
                            folders.insert(first_part.to_string());
                        } else if !entry_name.contains('/') {
                            files.insert(first_part.to_string());
                        }
                    }
                    
                    // Special check for libraries/ folder structure in ZIP
                    if entry_name.starts_with("libraries/") {
                        folders.insert("libraries".to_string());
                        if entry_name.contains("net/minecraftforge") {
                            files.insert("forge_marker".to_string());
                        }
                        if entry_name.contains("net/fabricmc") {
                            files.insert("fabric_marker".to_string());
                        }
                    }
                }
            }
        } else if extension == "7z" {
            let root = root_within_zip.as_deref().map(|r| {
                if r.ends_with('/') { r.to_string() } else { format!("{}/", r) }
            });

            sevenz_rust::SevenZReader::open(&path, "".into()).map_err(|e| e.to_string())?.for_each_entries(|entry, _| {
                let entry_name = entry.name().to_string();

                if let Some(ref root_path) = root {
                    if !entry_name.starts_with(root_path) {
                        return Ok(true);
                    }
                    let relative_name = entry_name.strip_prefix(root_path).unwrap_or(&entry_name);
                    if relative_name.is_empty() {
                        return Ok(true);
                    }

                    if let Some(first_part) = relative_name.split('/').next() {
                        if entry.is_directory() && !relative_name.trim_end_matches('/').contains('/') {
                            folders.insert(first_part.to_string());
                        } else if !relative_name.contains('/') {
                            files.insert(first_part.to_string());
                        }
                    }

                    if relative_name.starts_with("libraries/") {
                        folders.insert("libraries".to_string());
                        if relative_name.contains("net/minecraftforge") {
                            files.insert("forge_marker".to_string());
                        }
                        if relative_name.contains("net/fabricmc") {
                            files.insert("fabric_marker".to_string());
                        }
                    }
                } else {
                    if let Some(first_part) = entry_name.split('/').next() {
                        if entry.is_directory() {
                            folders.insert(first_part.to_string());
                        } else if !entry_name.contains('/') {
                            files.insert(first_part.to_string());
                        }
                    }
                    
                    if entry_name.starts_with("libraries/") {
                        folders.insert("libraries".to_string());
                        if entry_name.contains("net/minecraftforge") {
                            files.insert("forge_marker".to_string());
                        }
                        if entry_name.contains("net/fabricmc") {
                            files.insert("fabric_marker".to_string());
                        }
                    }
                }
                Ok(true)
            }).map_err(|e| e.to_string())?;
        }
    }

    // Detection Logic (Heuristics)
    
    // Quilt
    if files.iter().any(|f| f.to_lowercase().contains("quilt-server-launch.jar")) {
        return Ok("quilt".to_string());
    }

    // Fabric
    if files.iter().any(|f| f.to_lowercase().contains("fabric-server-launch.jar")) || files.contains("fabric_marker") {
        return Ok("fabric".to_string());
    }

    // Forge
    if files.iter().any(|f| f.to_lowercase().contains("forge") && f.to_lowercase().ends_with(".jar")) 
        || files.contains("user_jvm_args.txt") 
        || files.contains("forge_marker") 
    {
        return Ok("forge".to_string());
    }

    // Paper/Spigot/Bukkit
    if folders.contains("plugins") || files.contains("paper.yml") || files.contains("spigot.yml") || files.contains("bukkit.yml") {
        return Ok("paper".to_string());
    }

    // Vanilla
    if folders.contains("world") || files.contains("server.properties") {
        return Ok("vanilla".to_string());
    }

    Ok("unknown".to_string())
}

#[tauri::command]
pub async fn list_archive_contents(archive_path: String) -> Result<Vec<ZipEntry>, String> {
    let path = PathBuf::from(archive_path);
    if !path.is_file() {
        return Err("Path is not a file".to_string());
    }

    let extension = path.extension().map_or("", |ext| ext.to_str().unwrap_or("")).to_lowercase();
    let mut entries = Vec::new();

    if extension == "zip" {
        let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
        
        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name();
            
            entries.push(ZipEntry {
                name: name.split('/').filter(|s| !s.is_empty()).last().unwrap_or(name).to_string(),
                path: name.to_string(),
                is_dir: file.is_dir(),
            });
        }
    } else if extension == "7z" {
        sevenz_rust::SevenZReader::open(&path, "".into()).map_err(|e| e.to_string())?.for_each_entries(|entry, _| {
            let name = entry.name();
            entries.push(ZipEntry {
                name: name.split('/').filter(|s| !s.is_empty()).last().unwrap_or(name).to_string(),
                path: name.to_string(),
                is_dir: entry.is_directory(),
            });
            Ok(true)
        }).map_err(|e| e.to_string())?;
    } else {
        return Err(format!("Unsupported archive format: .{}", extension));
    }

    Ok(entries)
}
