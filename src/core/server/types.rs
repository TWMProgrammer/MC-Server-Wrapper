use serde::{Serialize, Deserialize};
use strum::Display;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Display, Serialize, Deserialize, Default)]
pub enum ServerStatus {
    #[default]
    Stopped,
    Starting,
    Installing,
    Running,
    Stopping,
    Crashed,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ResourceUsage {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub uptime: u64,
    pub player_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub current: u64,
    pub total: u64,
    pub message: String,
}
