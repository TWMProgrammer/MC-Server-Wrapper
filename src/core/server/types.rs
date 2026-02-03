use serde::Serialize;
use strum::Display;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Display, Serialize)]
pub enum ServerStatus {
    Stopped,
    Starting,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub current: u64,
    pub total: u64,
    pub message: String,
}
