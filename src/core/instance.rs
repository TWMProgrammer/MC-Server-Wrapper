use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use tokio::fs;
use chrono::{DateTime, Utc};
use tracing::info;
use super::scheduler::ScheduledTask;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LaunchMethod {
    StartupLine,
    BatFile,
}

impl Default for LaunchMethod {
    fn default() -> Self {
        Self::StartupLine
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CrashHandlingMode {
    Nothing,
    Elevated,
    Aggressive,
}

impl Default for CrashHandlingMode {
    fn default() -> Self {
        Self::Nothing
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceSettings {
    pub description: Option<String>,
    pub ram: u32,
    pub ram_unit: String, // "GB" or "MB"
    pub port: u16,
    pub force_save_all: bool,
    pub autostart: bool,
    pub java_path_override: Option<String>,
    pub launch_method: LaunchMethod,
    pub startup_line: String,
    pub bat_file: Option<String>,
    pub crash_handling: CrashHandlingMode,
    pub icon_path: Option<String>,
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            description: None,
            ram: 2,
            ram_unit: "GB".to_string(),
            port: 25565,
            force_save_all: false,
            autostart: false,
            java_path_override: None,
            launch_method: LaunchMethod::StartupLine,
            startup_line: "java -Xmx{ram}{unit} -jar server.jar nogui".to_string(),
            bat_file: None,
            crash_handling: CrashHandlingMode::Nothing,
            icon_path: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub mod_loader: Option<String>,
    pub loader_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub path: PathBuf,
    #[serde(default)]
    pub schedules: Vec<ScheduledTask>,
    #[serde(default)]
    pub settings: InstanceSettings,
}

pub struct InstanceManager {
    base_dir: PathBuf,
    registry_path: PathBuf,
}

impl InstanceManager {
    pub async fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).await?;
        }
        let registry_path = base_dir.join("instances.json");
        Ok(Self { base_dir, registry_path })
    }

    pub async fn create_instance(&self, name: &str, version: &str) -> Result<InstanceMetadata> {
        self.create_instance_full(name, version, None, None).await
    }

    pub async fn create_instance_full(&self, name: &str, version: &str, mod_loader: Option<String>, loader_version: Option<String>) -> Result<InstanceMetadata> {
        let id = Uuid::new_v4();
        let instance_path = self.base_dir.join(id.to_string());
        fs::create_dir_all(&instance_path).await?;

        let metadata = InstanceMetadata {
            id,
            name: name.to_string(),
            version: version.to_string(),
            mod_loader,
            loader_version,
            created_at: Utc::now(),
            last_run: None,
            path: instance_path,
            schedules: vec![],
            settings: InstanceSettings::default(),
        };

        let mut instances = self.list_instances().await?;
        instances.push(metadata.clone());
        self.save_registry(&instances).await?;

        info!("Created new instance: {} (ID: {})", name, id);
        Ok(metadata)
    }

    pub async fn import_instance<F>(&self, name: &str, source_path: PathBuf, jar_name: String, mod_loader: Option<String>, root_within_zip: Option<String>, on_progress: F) -> Result<InstanceMetadata> 
    where F: Fn(u64, u64, String) + Send + Sync + 'static
    {
        let id = Uuid::new_v4();
        let instance_path = self.base_dir.join(id.to_string());
        fs::create_dir_all(&instance_path).await?;

        if source_path.is_dir() {
            self.copy_dir_all(&source_path, &instance_path, on_progress).await?;
        } else if source_path.is_file() {
            let extension = source_path.extension().map_or("", |ext| ext.to_str().unwrap_or("")).to_lowercase();
            if extension == "zip" {
                self.extract_zip(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else if extension == "7z" {
                self.extract_7z(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else {
                return Err(anyhow::anyhow!("Unsupported archive format: .{}", extension));
            }
        } else {
            return Err(anyhow::anyhow!("Source path must be a directory or a supported archive file (.zip, .7z)"));
        }

        let mut settings = InstanceSettings::default();
        settings.startup_line = format!("java -Xmx{{ram}}{{unit}} -jar {} nogui", jar_name);

        let metadata = InstanceMetadata {
            id,
            name: name.to_string(),
            version: "Imported".to_string(),
            mod_loader,
            loader_version: None,
            created_at: Utc::now(),
            last_run: None,
            path: instance_path,
            schedules: vec![],
            settings,
        };

        let mut instances = self.list_instances().await?;
        instances.push(metadata.clone());
        self.save_registry(&instances).await?;

        info!("Imported instance: {} (ID: {})", name, id);
        Ok(metadata)
    }

    async fn extract_zip<F>(&self, zip_path: &Path, dst: &Path, root_within_zip: Option<String>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64, String) + Send + Sync + 'static
    {
        let zip_path = zip_path.to_path_buf();
        let dst = dst.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let file = std::fs::File::open(&zip_path)?;
            let mut archive = zip::ZipArchive::new(file)?;
            let total = archive.len() as u64;

            let root = root_within_zip.map(|r| {
                if r.ends_with('/') { r } else { format!("{}/", r) }
            });

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let name = file.name().to_string();
                
                on_progress(i as u64, total, format!("Extracting {}...", name));

                // If a root is specified, only extract files within that root
                if let Some(ref root_path) = root {
                    if !name.starts_with(root_path) {
                        continue;
                    }
                }

                let relative_name = if let Some(ref root_path) = root {
                    name.strip_prefix(root_path).unwrap_or(&name)
                } else {
                    &name
                };

                if relative_name.is_empty() {
                    continue;
                }

                let outpath = dst.join(relative_name);

                if name.ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)?;
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(&mut file, &mut outfile)?;
                }
            }
            Ok::<(), anyhow::Error>(())
        }).await?
    }

    async fn extract_7z<F>(&self, sz_path: &Path, dst: &Path, root_within_zip: Option<String>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64, String) + Send + Sync + 'static
    {
        let sz_path = sz_path.to_path_buf();
        let dst = dst.to_path_buf();

        tokio::task::spawn_blocking(move || {
            let root = root_within_zip.map(|r| {
                if r.ends_with('/') { r } else { format!("{}/", r) }
            });

            // For 7z we need to count entries first to have a total
            let total = {
                let mut file = std::fs::File::open(&sz_path)?;
                let len = file.metadata()?.len();
                let archive = sevenz_rust::Archive::read(&mut file, len, &[])
                    .map_err(|e| anyhow::anyhow!("7z read error: {}", e))?;
                archive.files.len() as u64
            };

            let mut current = 0;
            sevenz_rust::decompress_file_with_extract_fn(&sz_path, &dst, |entry, reader, out_dir| {
                let name = entry.name().to_string();
                current += 1;
                on_progress(current, total, format!("Extracting {}...", name));

                // If a root is specified, only extract files within that root
                if let Some(ref root_path) = root {
                    if !name.starts_with(root_path) {
                        return Ok(true); // Skip this entry but continue
                    }
                }

                let relative_name = if let Some(ref root_path) = root {
                    name.strip_prefix(root_path).unwrap_or(&name)
                } else {
                    &name
                };

                if relative_name.is_empty() {
                    return Ok(true);
                }

                let outpath = out_dir.join(relative_name);

                if entry.is_directory() {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            std::fs::create_dir_all(p)?;
                        }
                    }
                    let mut outfile = std::fs::File::create(&outpath)?;
                    std::io::copy(reader, &mut outfile)?;
                }
                Ok(true)
            }).map_err(|e| anyhow::anyhow!("7z decompression error: {}", e))?;

            Ok::<(), anyhow::Error>(())
        }).await?
    }

    pub async fn list_instances(&self) -> Result<Vec<InstanceMetadata>> {
        if !self.registry_path.exists() {
            return Ok(vec![]);
        }
        let content = fs::read_to_string(&self.registry_path).await?;
        let instances: Vec<InstanceMetadata> = serde_json::from_str(&content)
            .context("Failed to parse instances registry")?;
        Ok(instances)
    }

    pub async fn get_instance(&self, id: Uuid) -> Result<Option<InstanceMetadata>> {
        let instances = self.list_instances().await?;
        Ok(instances.into_iter().find(|i| i.id == id))
    }

    pub async fn delete_instance(&self, id: Uuid) -> Result<()> {
        let mut instances = self.list_instances().await?;
        if let Some(pos) = instances.iter().position(|i| i.id == id) {
            let instance = instances.remove(pos);
            if instance.path.exists() {
                fs::remove_dir_all(&instance.path).await?;
            }
            self.save_registry(&instances).await?;
            info!("Deleted instance: {} (ID: {})", instance.name, id);
        }
        Ok(())
    }

    pub async fn clone_instance(&self, id: Uuid, new_name: &str) -> Result<InstanceMetadata> {
        let instance = self.get_instance(id).await?
            .context("Instance not found")?;
        
        let new_id = Uuid::new_v4();
        let new_path = self.base_dir.join(new_id.to_string());
        
        // Copy directory recursively
        self.copy_dir_all(&instance.path, &new_path, |_, _, _| {}).await?;

        let new_metadata = InstanceMetadata {
            id: new_id,
            name: new_name.to_string(),
            version: instance.version.clone(),
            mod_loader: instance.mod_loader.clone(),
            loader_version: instance.loader_version.clone(),
            created_at: Utc::now(),
            last_run: None,
            path: new_path,
            schedules: instance.schedules.clone(),
            settings: instance.settings.clone(),
        };

        let mut instances = self.list_instances().await?;
        instances.push(new_metadata.clone());
        self.save_registry(&instances).await?;

        info!("Cloned instance: {} to {} (New ID: {})", instance.name, new_name, new_id);
        Ok(new_metadata)
    }

    async fn copy_dir_all<F>(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>, on_progress: F) -> Result<()> 
    where F: Fn(u64, u64, String) + Send + Sync + 'static
    {
        let src = src.as_ref().to_path_buf();
        let dst = dst.as_ref().to_path_buf();
        
        if !dst.exists() {
            fs::create_dir_all(&dst).await?;
        }

        let entries: Vec<_> = walkdir::WalkDir::new(&src).into_iter().filter_map(|e| e.ok()).collect();
        let total = entries.len() as u64;

        for (i, entry) in entries.into_iter().enumerate() {
            let relative_path = entry.path().strip_prefix(&src)?;
            let target_path = dst.join(relative_path);

            on_progress(i as u64, total, format!("Copying {}...", relative_path.display()));

            if entry.file_type().is_dir() {
                fs::create_dir_all(&target_path).await?;
            } else {
                fs::copy(entry.path(), &target_path).await?;
            }
        }
        Ok(())
    }

    pub async fn update_last_run(&self, id: Uuid) -> Result<()> {
        let mut instances = self.list_instances().await?;
        if let Some(instance) = instances.iter_mut().find(|i| i.id == id) {
            instance.last_run = Some(Utc::now());
            self.save_registry(&instances).await?;
        }
        Ok(())
    }

    pub async fn add_schedule(&self, instance_id: Uuid, task: ScheduledTask) -> Result<()> {
        let mut instances = self.list_instances().await?;
        if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
            instance.schedules.push(task);
            self.save_registry(&instances).await?;
        }
        Ok(())
    }

    pub async fn remove_schedule(&self, instance_id: Uuid, task_id: Uuid) -> Result<()> {
        let mut instances = self.list_instances().await?;
        if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
            instance.schedules.retain(|t| t.id != task_id);
            self.save_registry(&instances).await?;
        }
        Ok(())
    }

    pub async fn update_settings(&self, id: Uuid, name: Option<String>, settings: InstanceSettings) -> Result<()> {
        let mut instances = self.list_instances().await?;
        let updated = {
            if let Some(instance) = instances.iter_mut().find(|i| i.id == id) {
                if let Some(new_name) = name {
                    instance.name = new_name;
                }
                instance.settings = settings;
                Some(instance.name.clone())
            } else {
                None
            }
        };

        if let Some(instance_name) = updated {
            self.save_registry(&instances).await?;
            info!("Updated settings for instance: {} (ID: {})", instance_name, id);
        }
        Ok(())
    }

    pub fn get_base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }

    async fn save_registry(&self, instances: &[InstanceMetadata]) -> Result<()> {
        let content = serde_json::to_string_pretty(instances)?;
        fs::write(&self.registry_path, content).await?;
        Ok(())
    }
}

