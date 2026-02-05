use super::InstanceManager;
use crate::instance::types::InstanceMetadata;
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Row, sqlite::SqliteRow};
use std::path::PathBuf;
use tokio::fs;
use tracing::{error, info};
use uuid::Uuid;

impl InstanceManager {
    pub(crate) async fn migrate_from_json(&self) -> Result<()> {
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
        info!(
            "Migration complete. Old registry backed up to {:?}",
            backup_path
        );

        Ok(())
    }

    pub(crate) async fn save_instance_to_db(&self, instance: &InstanceMetadata) -> Result<()> {
        info!(
            "Saving instance to DB: {} (ID: {})",
            instance.name, instance.id
        );
        let settings_json = match serde_json::to_string(&instance.settings) {
            Ok(s) => s,
            Err(e) => {
                error!(
                    "Failed to serialize settings for instance {}: {}",
                    instance.name, e
                );
                return Err(e.into());
            }
        };
        let schedules_json = match serde_json::to_string(&instance.schedules) {
            Ok(s) => s,
            Err(e) => {
                error!(
                    "Failed to serialize schedules for instance {}: {}",
                    instance.name, e
                );
                return Err(e.into());
            }
        };

        match sqlx::query(
            "INSERT OR REPLACE INTO instances (id, name, version, mod_loader, loader_version, created_at, last_run, path, settings, schedules)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(instance.id.to_string())
        .bind(&instance.name)
        .bind(&instance.version)
        .bind(&instance.mod_loader)
        .bind(&instance.loader_version)
        .bind(instance.created_at.to_rfc3339())
        .bind(instance.last_run.map(|dt: chrono::DateTime<Utc>| dt.to_rfc3339()))
        .bind(instance.path.to_string_lossy().to_string())
        .bind(settings_json)
        .bind(schedules_json)
        .execute(self.db.pool())
        .await {
            Ok(_) => {
                info!("Successfully saved instance to DB: {}", instance.name);
                Ok(())
            },
            Err(e) => {
                error!("Failed to execute INSERT OR REPLACE query for instance {}: {}", instance.name, e);
                Err(anyhow::anyhow!("Failed to save instance to database: {}", e))
            }
        }
    }

    pub(crate) fn row_to_metadata(&self, row: SqliteRow) -> Result<InstanceMetadata> {
        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str)
            .context(format!("Failed to parse UUID from DB: {}", id_str))?;
        let name: String = row.try_get("name")?;
        let version: String = row.try_get("version")?;
        let mod_loader: Option<String> = row.try_get("mod_loader")?;
        let loader_version: Option<String> = row.try_get("loader_version")?;
        let created_at_str: String = row.try_get("created_at")?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .context(format!(
                "Failed to parse created_at for instance {}: {}",
                name, created_at_str
            ))?
            .with_timezone(&Utc);
        let last_run_str: Option<String> = row.try_get("last_run")?;
        let last_run = last_run_str.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        });
        let path: String = row.try_get("path")?;
        let settings_json: String = row.try_get("settings")?;
        let settings = serde_json::from_str(&settings_json).context(format!(
            "Failed to parse settings JSON for instance {}: {}",
            name, settings_json
        ))?;
        let schedules_json: String = row.try_get("schedules")?;
        let schedules = serde_json::from_str(&schedules_json).context(format!(
            "Failed to parse schedules JSON for instance {}: {}",
            name, schedules_json
        ))?;

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
            status: crate::server::types::ServerStatus::Stopped,
            ip: None,
            port: None,
            max_players: None,
            description: None,
        })
    }
}
