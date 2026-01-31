use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub jar_path: Option<PathBuf>,
    pub run_script: Option<String>,
    pub args: Vec<String>,
    pub java_path: Option<PathBuf>,
    pub max_memory: String,
    pub min_memory: String,
    pub auto_restart: bool,
    pub working_dir: PathBuf,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "Minecraft Server".to_string(),
            jar_path: Some(PathBuf::from("server.jar")),
            run_script: None,
            args: vec!["nogui".to_string()],
            java_path: None,
            max_memory: "2G".to_string(),
            min_memory: "1G".to_string(),
            auto_restart: true,
            working_dir: PathBuf::from("."),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub async fn load(&self) -> Result<ServerConfig> {
        if !self.config_path.exists() {
            let default_config = ServerConfig::default();
            self.save(&default_config).await?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path).await?;
        let config: ServerConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub async fn save(&self, config: &ServerConfig) -> Result<()> {
        let content = toml::to_string_pretty(config)?;
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&self.config_path, content).await?;
        Ok(())
    }
}
