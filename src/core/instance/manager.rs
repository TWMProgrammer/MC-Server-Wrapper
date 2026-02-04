use std::path::{Path, PathBuf};
use uuid::Uuid;
use anyhow::{Result, Context};
use tokio::fs;
use tracing::{info, warn};
use chrono::Utc;
use std::sync::Arc;
use sqlx::{sqlite::SqliteRow, Row};

use super::types::InstanceMetadata;
use super::archive::{extract_zip, extract_7z, copy_dir_all};
use super::types::InstanceSettings;
use crate::database::Database;

pub struct InstanceManager {
    pub(crate) base_dir: PathBuf,
    pub(crate) db: Arc<Database>,
}

impl InstanceManager {
    pub async fn new(base_dir: impl AsRef<Path>, db: Arc<Database>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            fs::create_dir_all(&base_dir).await?;
        }
        let manager = Self { base_dir, db };
        if let Err(e) = manager.migrate_from_json().await {
            warn!("Failed to migrate instances from JSON: {}", e);
        }
        Ok(manager)
    }

    async fn migrate_from_json(&self) -> Result<()> {
        let json_path = self.base_dir.join("instances.json");
        if !json_path.exists() {
            return Ok(());
        }

        info!("Migrating instances from instances.json to SQLite...");
        let content = fs::read_to_string(&json_path).await?;
        let instances: Vec<InstanceMetadata> = serde_json::from_str(&content)
            .context("Failed to parse instances.json during migration")?;

        for instance in instances {
            self.save_instance_to_db(&instance).await?;
        }

        // Rename the old file instead of deleting it, for safety
        let backup_path = self.base_dir.join("instances.json.bak");
        fs::rename(&json_path, &backup_path).await?;
        info!("Migration complete. Old registry backed up to {:?}", backup_path);

        Ok(())
    }

    async fn save_instance_to_db(&self, instance: &InstanceMetadata) -> Result<()> {
        let settings_json = serde_json::to_string(&instance.settings)?;
        let schedules_json = serde_json::to_string(&instance.schedules)?;

        sqlx::query(
            "INSERT OR REPLACE INTO instances (id, name, version, mod_loader, loader_version, created_at, last_run, path, settings, schedules)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(instance.id.to_string())
        .bind(&instance.name)
        .bind(&instance.version)
        .bind(&instance.mod_loader)
        .bind(&instance.loader_version)
        .bind(instance.created_at.to_rfc3339())
        .bind(instance.last_run.map(|dt| dt.to_rfc3339()))
        .bind(instance.path.to_string_lossy().to_string())
        .bind(settings_json)
        .bind(schedules_json)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    fn row_to_metadata(&self, row: SqliteRow) -> Result<InstanceMetadata> {
        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str)?;
        let name: String = row.try_get("name")?;
        let version: String = row.try_get("version")?;
        let mod_loader: Option<String> = row.try_get("mod_loader")?;
        let loader_version: Option<String> = row.try_get("loader_version")?;
        let created_at_str: String = row.try_get("created_at")?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc);
        let last_run_str: Option<String> = row.try_get("last_run")?;
        let last_run = last_run_str.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
        let path: String = row.try_get("path")?;
        let settings_json: String = row.try_get("settings")?;
        let settings = serde_json::from_str(&settings_json)?;
        let schedules_json: String = row.try_get("schedules")?;
        let schedules = serde_json::from_str(&schedules_json)?;

        Ok(InstanceMetadata {
            id,
            name,
            version,
            mod_loader,
            loader_version,
            created_at,
            last_run,
            path: PathBuf::from(path),
            settings,
            schedules,
        })
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

        self.save_instance_to_db(&metadata).await?;

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
            copy_dir_all(&source_path, &instance_path, on_progress).await?;
        } else if source_path.is_file() {
            let extension = source_path.extension().map_or("", |ext| ext.to_str().unwrap_or("")).to_lowercase();
            if extension == "zip" {
                extract_zip(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else if extension == "7z" {
                extract_7z(&source_path, &instance_path, root_within_zip, on_progress).await?;
            } else {
                return Err(anyhow::anyhow!("Unsupported archive format: .{}", extension));
            }
        } else {
            return Err(anyhow::anyhow!("Source path must be a directory or a supported archive file (.zip, .7z)"));
        }

        let mut settings = InstanceSettings::default();
        settings.startup_line = format!("java -Xmx{{ram}}{{unit}} -jar {} nogui", jar_name);

        // Check for server-icon.png
        let icon_path = instance_path.join("server-icon.png");
        if icon_path.exists() {
            settings.icon_path = Some(icon_path.to_string_lossy().to_string());
        }

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

        self.save_instance_to_db(&metadata).await?;

        info!("Imported instance: {} (ID: {})", name, id);
        Ok(metadata)
    }

    pub async fn list_instances(&self) -> Result<Vec<InstanceMetadata>> {
        let rows = sqlx::query("SELECT * FROM instances")
            .fetch_all(self.db.pool())
            .await?;

        let mut instances = Vec::new();
        for row in rows {
            instances.push(self.row_to_metadata(row)?);
        }
        Ok(instances)
    }

    pub async fn get_instance(&self, id: Uuid) -> Result<Option<InstanceMetadata>> {
        let row = sqlx::query("SELECT * FROM instances WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.db.pool())
            .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_metadata(row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_instance_by_name(&self, name: &str) -> Result<Option<InstanceMetadata>> {
        let row = sqlx::query("SELECT * FROM instances WHERE name = ?")
            .bind(name)
            .fetch_optional(self.db.pool())
            .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_metadata(row)?)),
            None => Ok(None),
        }
    }

    pub async fn delete_instance(&self, id: Uuid) -> Result<()> {
        if let Some(instance) = self.get_instance(id).await? {
            if instance.path.exists() {
                fs::remove_dir_all(&instance.path).await?;
            }
            sqlx::query("DELETE FROM instances WHERE id = ?")
                .bind(id.to_string())
                .execute(self.db.pool())
                .await?;
            info!("Deleted instance: {} (ID: {})", instance.name, id);
        }
        Ok(())
    }

    pub async fn delete_instance_by_name(&self, name: &str) -> Result<()> {
        if let Some(instance) = self.get_instance_by_name(name).await? {
            if instance.path.exists() {
                fs::remove_dir_all(&instance.path).await?;
            }
            sqlx::query("DELETE FROM instances WHERE name = ?")
                .bind(name)
                .execute(self.db.pool())
                .await?;
            info!("Deleted instance by name: {} (ID: {})", instance.name, instance.id);
        }
        Ok(())
    }

    pub async fn clone_instance(&self, id: Uuid, new_name: &str) -> Result<InstanceMetadata> {
        let instance = self.get_instance(id).await?
            .context("Instance not found")?;
        
        let new_id = Uuid::new_v4();
        let new_path = self.base_dir.join(new_id.to_string());
        
        // Copy directory recursively
        copy_dir_all(&instance.path, &new_path, |_, _, _| {}).await?;

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

        self.save_instance_to_db(&new_metadata).await?;

        info!("Cloned instance: {} to {} (New ID: {})", instance.name, new_name, new_id);
        Ok(new_metadata)
    }

    pub fn get_base_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }
}
