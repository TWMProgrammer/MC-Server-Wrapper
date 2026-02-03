use uuid::Uuid;
use anyhow::Result;
use tracing::info;
use chrono::Utc;
use super::manager::InstanceManager;
use super::types::InstanceSettings;
use super::super::scheduler::ScheduledTask;

impl InstanceManager {
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
}
