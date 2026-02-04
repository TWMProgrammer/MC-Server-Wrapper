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
    #[serde(default)]
    pub description: Option<String>,
    #[serde(alias = "ram", default = "default_min_ram")]
    pub min_ram: u32,
    #[serde(alias = "ram_unit", default = "default_ram_unit")]
    pub min_ram_unit: String,
    #[serde(default = "default_max_ram")]
    pub max_ram: u32,
    #[serde(default = "default_ram_unit")]
    pub max_ram_unit: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub force_save_all: bool,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub java_path_override: Option<String>,
    #[serde(default)]
    pub launch_method: LaunchMethod,
    #[serde(default = "default_startup_line")]
    pub startup_line: String,
    #[serde(default)]
    pub bat_file: Option<String>,
    #[serde(default)]
    pub crash_handling: CrashHandlingMode,
    #[serde(default)]
    pub icon_path: Option<String>,
}

fn default_min_ram() -> u32 { 1 }
fn default_max_ram() -> u32 { 2 }
fn default_ram_unit() -> String { "G".to_string() }
fn default_port() -> u16 { 25565 }
fn default_startup_line() -> String { "java -Xms{min_ram}{min_unit} -Xmx{max_ram}{max_unit} -jar server.jar nogui".to_string() }

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            description: None,
            min_ram: default_min_ram(),
            min_ram_unit: default_ram_unit(),
            max_ram: default_max_ram(),
            max_ram_unit: default_ram_unit(),
            port: default_port(),
            force_save_all: false,
            autostart: false,
            java_path_override: None,
            launch_method: LaunchMethod::StartupLine,
            startup_line: default_startup_line(),
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
