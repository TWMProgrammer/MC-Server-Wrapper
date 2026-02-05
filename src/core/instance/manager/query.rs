use super::InstanceManager;
use crate::instance::types::InstanceMetadata;
use crate::server_properties::read_server_properties;
use anyhow::Result;
use uuid::Uuid;

impl InstanceManager {
    pub async fn list_instances(&self) -> Result<Vec<InstanceMetadata>> {
        let rows = sqlx::query("SELECT * FROM instances")
            .fetch_all(self.db.pool())
            .await?;

        let mut instances = Vec::new();
        for row in rows {
            let mut metadata = self.row_to_metadata(row)?;
            let _ = self.enrich_metadata(&mut metadata).await; // Ignore errors for individual instances
            instances.push(metadata);
        }
        Ok(instances)
    }

    pub async fn get_instance(&self, id: Uuid) -> Result<Option<InstanceMetadata>> {
        let row = sqlx::query("SELECT * FROM instances WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.db.pool())
            .await?;

        match row {
            Some(row) => {
                let mut metadata = self.row_to_metadata(row)?;
                self.enrich_metadata(&mut metadata).await?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    pub async fn get_instance_by_name(&self, name: &str) -> Result<Option<InstanceMetadata>> {
        let row = sqlx::query("SELECT * FROM instances WHERE name = ?")
            .bind(name)
            .fetch_optional(self.db.pool())
            .await?;

        match row {
            Some(row) => {
                let mut metadata = self.row_to_metadata(row)?;
                self.enrich_metadata(&mut metadata).await?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    pub(crate) async fn enrich_metadata(&self, metadata: &mut InstanceMetadata) -> Result<()> {
        let props = match read_server_properties(&metadata.path).await {
            Ok(p) => p,
            Err(_) => return Ok(()),
        };
        if !props.is_empty() {
            if let Some(ip) = props.get("server-ip") {
                if !ip.trim().is_empty() {
                    metadata.ip = Some(ip.clone());
                }
            }
            if let Some(port_str) = props.get("server-port") {
                if let Ok(port) = port_str.parse::<u16>() {
                    metadata.port = Some(port);
                }
            }
            if let Some(max_players_str) = props.get("max-players") {
                if let Ok(max_players) = max_players_str.parse::<u32>() {
                    metadata.max_players = Some(max_players);
                }
            }
            if let Some(motd) = props.get("motd") {
                if !motd.trim().is_empty() {
                    metadata.description = Some(motd.clone());
                }
            }
        }
        Ok(())
    }
}
