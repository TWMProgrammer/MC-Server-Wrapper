use super::InstanceManager;
use tokio::fs;
use tracing::info;
use uuid::Uuid;
use anyhow::Result;

impl InstanceManager {
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
            info!(
                "Deleted instance by name: {} (ID: {})",
                instance.name, instance.id
            );
        }
        Ok(())
    }
}
