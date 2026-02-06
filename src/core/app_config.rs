use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManagedJavaVersion {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub version: String,
    pub major_version: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    // Interface
    pub display_ipv6: bool,
    pub hide_ip_address: bool,
    pub use_white_console_text: bool,

    // Navigation
    pub start_page: String, // "Dashboard", "Global Dashboard", etc.

    // Player List
    pub download_player_heads: bool,
    pub use_helm_heads: bool,
    pub query_heads_by_uuid: bool,

    // Server Tabs
    pub display_server_icon: bool,
    pub display_online_player_count: bool,
    pub display_server_version: bool,
    pub display_server_status: bool,
    pub display_navigational_buttons: bool,

    // Close Preference
    pub close_behavior: CloseBehavior,
    pub show_tray_notification: bool,

    // Appearance (Existing)
    pub accent_color: String,
    pub theme: String,
    pub scaling: f32,

    // Java Management
    #[serde(default)]
    pub managed_java_versions: Vec<ManagedJavaVersion>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum CloseBehavior {
    HideToSystemTray,
    HideToTaskbar,
    Exit,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            display_ipv6: false,
            hide_ip_address: false,
            use_white_console_text: false,
            start_page: "Dashboard".to_string(),
            download_player_heads: true,
            use_helm_heads: true,
            query_heads_by_uuid: false,
            display_server_icon: true,
            display_online_player_count: true,
            display_server_version: true,
            display_server_status: true,
            display_navigational_buttons: true,
            close_behavior: CloseBehavior::HideToSystemTray,
            show_tray_notification: true,
            accent_color: "Blue".to_string(),
            theme: "dark".to_string(),
            scaling: 1.0,
            managed_java_versions: vec![],
        }
    }
}

pub struct GlobalConfigManager {
    config_path: PathBuf,
}

impl GlobalConfigManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub async fn load(&self) -> Result<AppSettings> {
        if !self.config_path.exists() {
            let default_config = AppSettings::default();
            self.save(&default_config).await?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)
            .await
            .context("Failed to read app settings file")?;
        let config: AppSettings =
            serde_json::from_str(&content).context("Failed to parse app settings JSON")?;
        Ok(config)
    }

    pub async fn save(&self, config: &AppSettings) -> Result<()> {
        let content =
            serde_json::to_string_pretty(config).context("Failed to serialize app settings")?;
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create config directory")?;
        }
        fs::write(&self.config_path, content)
            .await
            .context("Failed to write app settings file")?;
        Ok(())
    }
}
