use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use super::super::scheduler::ScheduledTask;
use super::super::server::types::ServerStatus;

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
    pub ram_unit: String, // "G" or "M"
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
            ram_unit: "G".to_string(),
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
    #[serde(default)]
    pub status: ServerStatus,
    // Dynamic properties from server.properties
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub max_players: Option<u32>,
    #[serde(default)]
    pub description: Option<String>,
}
