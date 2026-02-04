use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Instance error: {0}")]
    Instance(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Mod error: {0}")]
    Mod(String),

    #[error("Backup error: {0}")]
    Backup(String),

    #[error("Scheduler error: {0}")]
    Scheduler(String),

    #[error("Java error: {0}")]
    Java(String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Config(err.to_string())
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::Validation(format!("Invalid UUID: {}", err))
    }
}

// Helper to convert any result to Result<T, AppError> using anyhow as bridge
pub trait ToAppResult<T> {
    fn to_app_result(self) -> Result<T, AppError>;
}

impl<T, E> ToAppResult<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn to_app_result(self) -> Result<T, AppError> {
        self.map_err(Into::into)
    }
}
