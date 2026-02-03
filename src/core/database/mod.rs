use std::path::Path;
use anyhow::{Result, Context};
use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};
use std::str::FromStr;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.display()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePool::connect_with(options).await
            .context("Failed to connect to SQLite database")?;

        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS instances (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                mod_loader TEXT,
                loader_version TEXT,
                created_at TEXT NOT NULL,
                last_run TEXT,
                path TEXT NOT NULL,
                settings TEXT NOT NULL,
                schedules TEXT NOT NULL
            )"
        )
        .execute(&self.pool)
        .await
        .context("Failed to create instances table")?;

        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
