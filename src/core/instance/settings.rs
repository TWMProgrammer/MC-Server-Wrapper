use uuid::Uuid;
use anyhow::{Result, Context};
use tracing::info;
use chrono::Utc;
use super::manager::InstanceManager;
use super::types::InstanceSettings;
use super::super::scheduler::ScheduledTask;
use super::super::server_properties::{read_server_properties, write_server_properties};

impl InstanceManager {
    pub async fn update_last_run(&self, id: Uuid) -> Result<()> {
        let last_run = Utc::now().to_rfc3339();
        
        sqlx::query("UPDATE instances SET last_run = ? WHERE id = ?")
            .bind(last_run)
            .bind(id.to_string())
            .execute(self.db.pool())
            .await?;
            
        Ok(())
    }

    pub async fn add_schedule(&self, instance_id: Uuid, task: ScheduledTask) -> Result<()> {
        let mut metadata = self.get_instance(instance_id).await?
            .context("Instance not found")?;
        
        metadata.schedules.push(task);
        let schedules_json = serde_json::to_string(&metadata.schedules)?;

        sqlx::query("UPDATE instances SET schedules = ? WHERE id = ?")
            .bind(schedules_json)
            .bind(instance_id.to_string())
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    pub async fn remove_schedule(&self, instance_id: Uuid, task_id: Uuid) -> Result<()> {
        let mut metadata = self.get_instance(instance_id).await?
            .context("Instance not found")?;
        
        metadata.schedules.retain(|t| t.id != task_id);
        let schedules_json = serde_json::to_string(&metadata.schedules)?;

        sqlx::query("UPDATE instances SET schedules = ? WHERE id = ?")
            .bind(schedules_json)
            .bind(instance_id.to_string())
            .execute(self.db.pool())
            .await?;

        Ok(())
    }

    pub async fn update_settings(&self, id: Uuid, name: Option<String>, settings: InstanceSettings) -> Result<()> {
        let instance = self.get_instance(id).await?
            .context("Instance not found")?;

        let settings_json = serde_json::to_string(&settings)?;
        
        // Update server.properties if it exists
        let mut props = read_server_properties(&instance.path).await?;
        let mut props_changed = false;

        let port_str = settings.port.to_string();
        if props.get("server-port") != Some(&port_str) {
            props.insert("server-port".to_string(), port_str);
            props_changed = true;
        }

        if props_changed {
            write_server_properties(&instance.path, &props).await?;
        }

        if let Some(new_name) = name {
            sqlx::query("UPDATE instances SET name = ?, settings = ? WHERE id = ?")
                .bind(&new_name)
                .bind(settings_json)
                .bind(id.to_string())
                .execute(self.db.pool())
                .await?;
            info!("Updated settings and name for instance: {} (ID: {})", new_name, id);
        } else {
            sqlx::query("UPDATE instances SET settings = ? WHERE id = ?")
                .bind(settings_json)
                .bind(id.to_string())
                .execute(self.db.pool())
                .await?;
            info!("Updated settings for instance (ID: {})", id);
        }
        
        Ok(())
    }
}
