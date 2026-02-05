use mc_server_wrapper_core::instance::{InstanceManager, InstanceMetadata};
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Emitter, State};

use super::super::{AppError, CommandResult};

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
#[allow(non_snake_case)]
pub async fn import_instance(
    app_handle: tauri::AppHandle,
    instance_manager: State<'_, Arc<InstanceManager>>,
    name: String,
    sourcePath: String,
    jarName: String,
    serverType: String,
    rootWithinZip: Option<String>,
    scriptPath: Option<String>,
) -> CommandResult<InstanceMetadata> {
    let path = PathBuf::from(sourcePath);
    let mod_loader = if serverType == "vanilla" || serverType == "custom" {
        None
    } else {
        Some(serverType)
    };

    let app_handle_clone = app_handle.clone();
    instance_manager
        .import_instance(
            &name,
            path,
            jarName,
            mod_loader,
            rootWithinZip,
            scriptPath,
            move |current, total, message| {
                let _ = app_handle_clone.emit(
                    "import-progress",
                    ImportProgressPayload {
                        current,
                        total,
                        message,
                    },
                );
            },
        )
        .await
        .map_err(AppError::from)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn list_jars_in_source(
    sourcePath: String,
    rootWithinZip: Option<String>,
) -> CommandResult<Vec<String>> {
    let path = PathBuf::from(&sourcePath);
    let mut jars = Vec::new();

    if path.is_dir() {
        let mut entries = tokio::fs::read_dir(&path).await.map_err(AppError::from)?;
        while let Some(entry) = entries.next_entry().await.map_err(AppError::from)? {
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
        let extension = path
            .extension()
            .map_or("", |ext| ext.to_str().unwrap_or(""))
            .to_lowercase();
        if extension == "zip" {
            let file = std::fs::File::open(&path).map_err(AppError::from)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;

            let root = rootWithinZip.map(|r| {
                if r.ends_with('/') {
                    r
                } else {
                    format!("{}/", r)
                }
            });

            for i in 0..archive.len() {
                let file = archive
                    .by_index(i)
                    .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;
                let name = file.name();

                if let Some(ref root_path) = root {
                    if !name.starts_with(root_path) {
                        continue;
                    }
                    let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                    if !file.is_dir()
                        && relative_name.to_lowercase().ends_with(".jar")
                        && !relative_name.contains('/')
                    {
                        jars.push(relative_name.to_string());
                    }
                } else if !file.is_dir() && name.to_lowercase().ends_with(".jar") {
                    jars.push(name.to_string());
                }
            }
        } else if extension == "7z" {
            let root = rootWithinZip.as_deref().map(|r| {
                if r.ends_with('/') {
                    r.to_string()
                } else {
                    format!("{}/", r)
                }
            });

            sevenz_rust::SevenZReader::open(&path, "".into())
                .map_err(|e| AppError::Config(e.to_string()))?
                .for_each_entries(|entry, _| {
                    let name = entry.name();
                    if let Some(ref root_path) = root {
                        if !name.starts_with(root_path) {
                            return Ok(true);
                        }
                        let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                        if !entry.is_directory()
                            && relative_name.to_lowercase().ends_with(".jar")
                            && !relative_name.contains('/')
                        {
                            jars.push(relative_name.to_string());
                        }
                    } else if !entry.is_directory() && name.to_lowercase().ends_with(".jar") {
                        jars.push(name.to_string());
                    }
                    Ok(true)
                })
                .map_err(|e| AppError::Config(e.to_string()))?;
        }
    }

    Ok(jars)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn list_scripts_in_source(
    sourcePath: String,
    rootWithinZip: Option<String>,
) -> CommandResult<Vec<String>> {
    let path = PathBuf::from(&sourcePath);
    let mut scripts = Vec::new();

    if path.is_dir() {
        let mut entries = tokio::fs::read_dir(&path).await.map_err(AppError::from)?;
        while let Some(entry) = entries.next_entry().await.map_err(AppError::from)? {
            let entry_path = entry.path();
            if entry_path.is_file() {
                if let Some(extension) = entry_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if ext == "bat" || ext == "cmd" || ext == "sh" {
                        if let Some(file_name) = entry_path.file_name() {
                            scripts.push(file_name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    } else if path.is_file() {
        let extension = path
            .extension()
            .map_or("", |ext| ext.to_str().unwrap_or(""))
            .to_lowercase();
        if extension == "zip" {
            let file = std::fs::File::open(&path).map_err(AppError::from)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;

            let root = rootWithinZip.map(|r| {
                if r.ends_with('/') {
                    r
                } else {
                    format!("{}/", r)
                }
            });

            for i in 0..archive.len() {
                let file = archive
                    .by_index(i)
                    .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;
                let name = file.name();

                if let Some(ref root_path) = root {
                    if !name.starts_with(root_path) {
                        continue;
                    }
                    let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                    let lower_name = relative_name.to_lowercase();
                    if !file.is_dir()
                        && (lower_name.ends_with(".bat")
                            || lower_name.ends_with(".cmd")
                            || lower_name.ends_with(".sh"))
                        && !relative_name.contains('/')
                    {
                        scripts.push(relative_name.to_string());
                    }
                } else if !file.is_dir() {
                    let lower_name = name.to_lowercase();
                    if lower_name.ends_with(".bat")
                        || lower_name.ends_with(".cmd")
                        || lower_name.ends_with(".sh")
                    {
                        scripts.push(name.to_string());
                    }
                }
            }
        } else if extension == "7z" {
            let root = rootWithinZip.as_deref().map(|r| {
                if r.ends_with('/') {
                    r.to_string()
                } else {
                    format!("{}/", r)
                }
            });

            sevenz_rust::SevenZReader::open(&path, "".into())
                .map_err(|e| AppError::Config(e.to_string()))?
                .for_each_entries(|entry, _| {
                    let name = entry.name();
                    if let Some(ref root_path) = root {
                        if !name.starts_with(root_path) {
                            return Ok(true);
                        }
                        let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                        let lower_name = relative_name.to_lowercase();
                        if !entry.is_directory()
                            && (lower_name.ends_with(".bat")
                                || lower_name.ends_with(".cmd")
                                || lower_name.ends_with(".sh"))
                            && !relative_name.contains('/')
                        {
                            scripts.push(relative_name.to_string());
                        }
                    } else if !entry.is_directory() {
                        let lower_name = name.to_lowercase();
                        if lower_name.ends_with(".bat")
                            || lower_name.ends_with(".cmd")
                            || lower_name.ends_with(".sh")
                        {
                            scripts.push(name.to_string());
                        }
                    }
                    Ok(true)
                })
                .map_err(|e| AppError::Config(e.to_string()))?;
        }
    }

    Ok(scripts)
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn check_server_properties_exists(
    sourcePath: String,
    rootWithinZip: Option<String>,
) -> CommandResult<bool> {
    let path = PathBuf::from(&sourcePath);

    if path.is_dir() {
        Ok(path.join("server.properties").exists())
    } else if path.is_file() {
        let extension = path
            .extension()
            .map_or("", |ext| ext.to_str().unwrap_or(""))
            .to_lowercase();
        if extension == "zip" {
            let file = std::fs::File::open(&path).map_err(AppError::from)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;

            let target = if let Some(root) = rootWithinZip {
                let root = if root.ends_with('/') {
                    root
                } else {
                    format!("{}/", root)
                };
                format!("{}server.properties", root)
            } else {
                "server.properties".to_string()
            };

            for i in 0..archive.len() {
                let file = archive
                    .by_index(i)
                    .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;
                if file.name() == target {
                    return Ok(true);
                }
            }
            Ok(false)
        } else if extension == "7z" {
            let target = if let Some(root) = rootWithinZip {
                let root = if root.ends_with('/') {
                    root
                } else {
                    format!("{}/", root)
                };
                format!("{}server.properties", root)
            } else {
                "server.properties".to_string()
            };

            let mut exists = false;
            sevenz_rust::SevenZReader::open(&path, "".into())
                .map_err(|e| AppError::Config(e.to_string()))?
                .for_each_entries(|entry, _| {
                    if entry.name() == target {
                        exists = true;
                        return Ok(false); // Stop iterating
                    }
                    Ok(true)
                })
                .map_err(|e| AppError::Config(e.to_string()))?;
            Ok(exists)
        } else {
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

#[tauri::command]
#[allow(non_snake_case)]
pub async fn detect_server_type(
    sourcePath: String,
    rootWithinZip: Option<String>,
) -> CommandResult<String> {
    let path = PathBuf::from(&sourcePath);
    let mut files = HashSet::new();
    let mut folders = HashSet::new();

    if path.is_dir() {
        let mut entries = tokio::fs::read_dir(&path).await.map_err(AppError::from)?;
        while let Some(entry) = entries.next_entry().await.map_err(AppError::from)? {
            let entry_path = entry.path();
            if let Some(name) = entry_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
            {
                if entry_path.is_dir() {
                    folders.insert(name);
                } else {
                    files.insert(name);
                }
            }
        }
    } else if path.is_file() {
        let extension = path
            .extension()
            .map_or("", |ext| ext.to_str().unwrap_or(""))
            .to_lowercase();
        if extension == "zip" {
            let file = std::fs::File::open(&path).map_err(AppError::from)?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;

            let root = rootWithinZip.map(|r| {
                if r.ends_with('/') {
                    r
                } else {
                    format!("{}/", r)
                }
            });

            for i in 0..archive.len() {
                let file = archive
                    .by_index(i)
                    .map_err(|e: zip::result::ZipError| AppError::Config(e.to_string()))?;
                let name = file.name();

                if let Some(ref root_path) = root {
                    if !name.starts_with(root_path) {
                        continue;
                    }
                    let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                    if file.is_dir() {
                        if let Some(folder_name) = relative_name.split('/').next() {
                            if !folder_name.is_empty() {
                                folders.insert(folder_name.to_string());
                            }
                        }
                    } else if !relative_name.contains('/') {
                        files.insert(relative_name.to_string());
                    }

                    if relative_name.starts_with("libraries/") {
                        folders.insert("libraries".to_string());
                        if relative_name.contains("net/minecraftforge") {
                            files.insert("forge_marker".to_string());
                        }
                        if relative_name.contains("net/fabricmc") {
                            files.insert("fabric_marker".to_string());
                        }
                        if relative_name.contains("org/quiltmc") {
                            files.insert("quilt_marker".to_string());
                        }
                    }
                } else {
                    if file.is_dir() {
                        if let Some(folder_name) = name.split('/').next() {
                            if !folder_name.is_empty() {
                                folders.insert(folder_name.to_string());
                            }
                        }
                    } else if !name.contains('/') {
                        files.insert(name.to_string());
                    }

                    if name.starts_with("libraries/") {
                        folders.insert("libraries".to_string());
                        if name.contains("net/minecraftforge") {
                            files.insert("forge_marker".to_string());
                        }
                        if name.contains("net/fabricmc") {
                            files.insert("fabric_marker".to_string());
                        }
                        if name.contains("org/quiltmc") {
                            files.insert("quilt_marker".to_string());
                        }
                    }
                }
            }
        } else if extension == "7z" {
            let root = rootWithinZip.map(|r| {
                if r.ends_with('/') {
                    r
                } else {
                    format!("{}/", r)
                }
            });

            sevenz_rust::SevenZReader::open(&path, "".into())
                .map_err(|e| AppError::Config(e.to_string()))?
                .for_each_entries(|entry, _| {
                    let name = entry.name();
                    if let Some(ref root_path) = root {
                        if name.starts_with(root_path) {
                            let relative_name = name.strip_prefix(root_path).unwrap_or(name);
                            if entry.is_directory() {
                                if let Some(folder_name) = relative_name.split('/').next() {
                                    if !folder_name.is_empty() {
                                        folders.insert(folder_name.to_string());
                                    }
                                }
                            } else if !relative_name.contains('/') {
                                files.insert(relative_name.to_string());
                            }

                            if relative_name.starts_with("libraries/") {
                                folders.insert("libraries".to_string());
                                if relative_name.contains("net/minecraftforge") {
                                    files.insert("forge_marker".to_string());
                                }
                                if relative_name.contains("net/fabricmc") {
                                    files.insert("fabric_marker".to_string());
                                }
                                if relative_name.contains("org/quiltmc") {
                                    files.insert("quilt_marker".to_string());
                                }
                            }
                        }
                    } else {
                        if entry.is_directory() {
                            if let Some(folder_name) = name.split('/').next() {
                                if !folder_name.is_empty() {
                                    folders.insert(folder_name.to_string());
                                }
                            }
                        } else if !name.contains('/') {
                            files.insert(name.to_string());
                        }

                        if name.starts_with("libraries/") {
                            folders.insert("libraries".to_string());
                            if name.contains("net/minecraftforge") {
                                files.insert("forge_marker".to_string());
                            }
                            if name.contains("net/fabricmc") {
                                files.insert("fabric_marker".to_string());
                            }
                            if name.contains("org/quiltmc") {
                                files.insert("quilt_marker".to_string());
                            }
                        }
                    }
                    Ok(true)
                })
                .map_err(|e| AppError::Config(e.to_string()))?;
        }
    }

    // Detection Logic (Heuristics)

    // Quilt
    if files.iter().any(|f| {
        let fl = f.to_lowercase();
        fl.contains("quilt-server-launch.jar") || fl.contains("quilt-server.jar")
    }) || files.contains("quilt_marker")
    {
        return Ok("quilt".to_string());
    }

    // Fabric
    if files.iter().any(|f| {
        let fl = f.to_lowercase();
        fl.contains("fabric-server-launch.jar") || fl.contains("fabric-server.jar")
    }) || files.contains("fabric_marker")
    {
        return Ok("fabric".to_string());
    }

    // Forge
    if files
        .iter()
        .any(|f| f.to_lowercase().contains("forge") && f.to_lowercase().ends_with(".jar"))
        || files.contains("user_jvm_args.txt")
        || files.contains("forge_marker")
    {
        return Ok("forge".to_string());
    }

    // Paper/Spigot/Bukkit
    if folders.contains("plugins")
        || files.contains("paper.yml")
        || files.contains("spigot.yml")
        || files.contains("bukkit.yml")
    {
        return Ok("paper".to_string());
    }

    // Vanilla
    if folders.contains("world") || files.contains("server.properties") {
        return Ok("vanilla".to_string());
    }

    Ok("unknown".to_string())
}

#[tauri::command]
pub async fn list_archive_contents(archive_path: String) -> CommandResult<Vec<ZipEntry>> {
    let path = PathBuf::from(archive_path);
    if !path.is_file() {
        return Err(AppError::Validation("Path is not a file".to_string()));
    }

    let extension = path
        .extension()
        .map_or("", |ext| ext.to_str().unwrap_or(""))
        .to_lowercase();
    let mut entries = Vec::new();

    if extension == "zip" {
        let file = std::fs::File::open(&path).map_err(AppError::from)?;
        let mut archive =
            zip::ZipArchive::new(file).map_err(|e| AppError::Config(e.to_string()))?;

        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| AppError::Config(e.to_string()))?;
            let name = file.name();

            entries.push(ZipEntry {
                name: name
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .last()
                    .unwrap_or(name)
                    .to_string(),
                path: name.to_string(),
                is_dir: file.is_dir(),
            });
        }
    } else if extension == "7z" {
        sevenz_rust::SevenZReader::open(&path, "".into())
            .map_err(|e| AppError::Config(e.to_string()))?
            .for_each_entries(|entry, _| {
                let name = entry.name();
                entries.push(ZipEntry {
                    name: name
                        .split('/')
                        .filter(|s| !s.is_empty())
                        .last()
                        .unwrap_or(name)
                        .to_string(),
                    path: name.to_string(),
                    is_dir: entry.is_directory(),
                });
                Ok(true)
            })
            .map_err(|e| AppError::Config(e.to_string()))?;
    } else {
        return Err(AppError::Validation(format!(
            "Unsupported archive format: .{}",
            extension
        )));
    }

    Ok(entries)
}
