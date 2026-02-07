use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{Column as _, Row as _, SqlitePool, TypeInfo as _, ValueRef as _};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub primary_key: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum DatabaseType {
    SQLite,
    H2,
    SQL,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseFile {
    pub name: String,
    pub path: PathBuf,
    pub db_type: DatabaseType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseGroup {
    pub name: String,
    pub files: Vec<DatabaseFile>,
}

/// Scans for database files in the given root directory and groups them by their parent folder.
pub fn find_database_files(root: &Path) -> Vec<DatabaseGroup> {
    use std::collections::HashMap;
    let mut groups: HashMap<String, Vec<DatabaseFile>> = HashMap::new();

    info!("Scanning for databases in: {:?}", root);

    // 1. Scan root level for databases (e.g. world-specific DBs if any, or general server DBs)
    for entry in std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            let path = entry.path();
            if let Some(db_type) = get_db_type(&path) {
                let db_file = DatabaseFile {
                    name: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    path: path.to_path_buf(),
                    db_type,
                };
                groups
                    .entry("Server Root".to_string())
                    .or_default()
                    .push(db_file);
            }
        }
    }

    // 2. Scan plugins directory
    let plugins_root = root.join("plugins");
    if plugins_root.exists() {
        info!("Scanning plugins directory: {:?}", plugins_root);
        for entry in WalkDir::new(&plugins_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(db_type) = get_db_type(&path) {
                    let relative_to_plugins = path.strip_prefix(&plugins_root).unwrap_or(path);
                    let components: Vec<_> = relative_to_plugins.components().collect();

                    let group_name = if components.len() > 1 {
                        // The first component of the path relative to 'plugins/' is the plugin folder name
                        components[0].as_os_str().to_string_lossy().to_string()
                    } else {
                        // File is directly in the 'plugins/' folder
                        "General Plugins".to_string()
                    };

                    let db_file = DatabaseFile {
                        name: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string(),
                        path: path.to_path_buf(),
                        db_type,
                    };

                    groups.entry(group_name).or_default().push(db_file);
                }
            }
        }
    } else {
        warn!("Plugins directory not found at {:?}", plugins_root);
    }

    let mut result: Vec<DatabaseGroup> = groups
        .into_iter()
        .map(|(name, files)| DatabaseGroup { name, files })
        .collect();

    // Sort groups: "Server Root" first, then others alphabetically
    result.sort_by(|a, b| {
        if a.name == "Server Root" {
            std::cmp::Ordering::Less
        } else if b.name == "Server Root" {
            std::cmp::Ordering::Greater
        } else {
            a.name.cmp(&b.name)
        }
    });

    info!("Found {} database groups", result.len());
    result
}

fn get_db_type(path: &Path) -> Option<DatabaseType> {
    let path_str = path.to_string_lossy().to_lowercase();

    // Exclude trace files - these are logs, not databases
    if path_str.ends_with(".trace.db") {
        return None;
    }

    // H2 databases
    if path_str.ends_with(".mv.db") || path_str.ends_with(".h2.db") {
        return Some(DatabaseType::H2);
    }

    // SQL Scripts (H2 exports or general SQL)
    if path_str.ends_with(".h2.sql") || path_str.ends_with(".sql") {
        return Some(DatabaseType::SQL);
    }

    // SQLite or generic DB files
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "sqlite" | "sqlite3" => return Some(DatabaseType::SQLite),
            "db" => {
                // If it's just .db, it's usually SQLite in the Minecraft world (e.g., CoreProtect, LuckPerms)
                return Some(DatabaseType::SQLite);
            }
            _ => {}
        }
    }

    None
}

/// Reads the content of a SQL script file.
pub async fn read_sql_file(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

fn parse_h2_sql(content: &str) -> HashMap<String, TableData> {
    let mut tables = HashMap::new();
    let mut temp_data: HashMap<String, Vec<Vec<serde_json::Value>>> = HashMap::new();
    let mut table_columns: HashMap<String, Vec<String>> = HashMap::new();

    // 1. Parse temporary data: INSERT INTO O_... VALUES(...);
    let insert_values_re = Regex::new(r"(?i)INSERT INTO (O_\d+) VALUES\((.*?)\);").unwrap();
    for cap in insert_values_re.captures_iter(content) {
        let temp_name = cap[1].to_string();
        let values_str = &cap[2];

        // Simple CSV-ish parser for values (handles strings in single quotes)
        let mut row = Vec::new();
        let mut current_val = String::new();
        let mut in_quotes = false;
        let mut chars = values_str.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '\'' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    let val = current_val.trim();
                    row.push(parse_sql_value(val));
                    current_val.clear();
                }
                _ => current_val.push(c),
            }
        }
        if !current_val.is_empty() {
            row.push(parse_sql_value(current_val.trim()));
        }

        temp_data.entry(temp_name).or_default().push(row);
    }

    // 2. Parse table definitions: CREATE CACHED TABLE "PUBLIC"."TABLE_NAME"(...)
    let create_table_re =
        Regex::new(r#"(?is)CREATE CACHED TABLE "PUBLIC"\."(.*?)"\s*\((.*?)\)\;"#).unwrap();
    for cap in create_table_re.captures_iter(content) {
        let table_name = cap[1].to_string();
        let columns_block = &cap[2];

        let mut columns = Vec::new();
        // Extract column names: "COL_NAME" TYPE ...
        let col_re = Regex::new(r#""(.*?)"\s+[A-Z]+"#).unwrap();
        for col_cap in col_re.captures_iter(columns_block) {
            columns.push(col_cap[1].to_string());
        }
        table_columns.insert(table_name, columns);
    }

    // 3. Map temp data to real tables: INSERT INTO "PUBLIC"."TABLE_NAME" SELECT * FROM O_...;
    let map_data_re =
        Regex::new(r#"(?i)INSERT INTO "PUBLIC"\."(.*?)" SELECT \* FROM (O_\d+);"#).unwrap();
    for cap in map_data_re.captures_iter(content) {
        let table_name = cap[1].to_string();
        let temp_name = cap[2].to_string();

        if let (Some(columns), Some(rows)) =
            (table_columns.get(&table_name), temp_data.get(&temp_name))
        {
            tables.insert(
                table_name,
                TableData {
                    columns: columns.clone(),
                    rows: rows.clone(),
                },
            );
        }
    }

    // 4. Handle direct inserts: INSERT INTO "PUBLIC"."TABLE_NAME" VALUES(...);
    let direct_insert_re =
        Regex::new(r#"(?i)INSERT INTO "PUBLIC"\."(.*?)" VALUES\((.*?)\);"#).unwrap();
    for cap in direct_insert_re.captures_iter(content) {
        let table_name = cap[1].to_string();
        let values_str = &cap[2];

        // (Similar value parsing as above...)
        let mut row = Vec::new();
        let mut current_val = String::new();
        let mut in_quotes = false;
        let mut chars = values_str.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\'' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    row.push(parse_sql_value(current_val.trim()));
                    current_val.clear();
                }
                _ => current_val.push(c),
            }
        }
        if !current_val.is_empty() {
            row.push(parse_sql_value(current_val.trim()));
        }

        if let Some(data) = tables.get_mut(&table_name) {
            data.rows.push(row);
        } else if let Some(columns) = table_columns.get(&table_name) {
            tables.insert(
                table_name.clone(),
                TableData {
                    columns: columns.clone(),
                    rows: vec![row],
                },
            );
        }
    }

    tables
}

fn parse_sql_value(val: &str) -> serde_json::Value {
    if val.to_uppercase() == "NULL" {
        serde_json::Value::Null
    } else if val.starts_with('\'') && val.ends_with('\'') {
        serde_json::Value::String(val[1..val.len() - 1].to_string())
    } else if let Ok(n) = val.parse::<i64>() {
        serde_json::Value::Number(n.into())
    } else if let Ok(f) = val.parse::<f64>() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::String(val.to_string()))
    } else {
        serde_json::Value::String(val.to_string())
    }
}

/// Lists all tables in the database at the given path.
pub async fn list_tables(path: &Path) -> Result<Vec<String>> {
    let db_type = get_db_type(path).ok_or_else(|| anyhow::anyhow!("Unsupported database file"))?;

    match db_type {
        DatabaseType::SQLite => {
            let pool = get_connection(path).await?;
            let rows = sqlx::query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            )
            .fetch_all(&pool)
            .await?;

            let tables = rows.iter().map(|row| row.get::<String, _>(0)).collect();
            Ok(tables)
        }
        DatabaseType::H2 => {
            // For H2, we'd need a separate H2 driver or use a shell command to list tables.
            // For now, return an empty list or a message.
            Ok(vec!["(H2 tables listing not yet implemented)".to_string()])
        }
        DatabaseType::SQL => {
            let content = std::fs::read_to_string(path)?;
            let tables = parse_h2_sql(&content);
            let mut table_names: Vec<String> = tables.keys().cloned().collect();
            table_names.sort();
            Ok(table_names)
        }
    }
}

/// Gets a page of data from a table.
pub async fn get_table_data(
    path: &Path,
    table: &str,
    limit: u32,
    offset: u32,
) -> Result<TableData> {
    let db_type = get_db_type(path).ok_or_else(|| anyhow::anyhow!("Unsupported database file"))?;

    if db_type == DatabaseType::SQL {
        let content = std::fs::read_to_string(path)?;
        let tables = parse_h2_sql(&content);
        let table_data = tables
            .get(table)
            .ok_or_else(|| anyhow::anyhow!("Table not found in SQL script"))?;

        let start = offset as usize;
        let end = (offset + limit) as usize;
        let rows_len = table_data.rows.len();
        let paginated_rows = if start >= rows_len {
            Vec::new()
        } else {
            table_data.rows[start..std::cmp::min(end, rows_len)].to_vec()
        };

        return Ok(TableData {
            columns: table_data.columns.clone(),
            rows: paginated_rows,
        });
    }

    if db_type != DatabaseType::SQLite {
        return Err(anyhow::anyhow!(
            "Data inspection is only supported for SQLite databases and SQL scripts."
        ));
    }

    // Basic validation to prevent SQL injection for table names
    if !table.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(anyhow::anyhow!("Invalid table name: {}", table));
    }

    let pool = get_connection(path).await?;

    let query_str = format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, limit, offset);
    let rows = sqlx::query(&query_str).fetch_all(&pool).await?;

    if rows.is_empty() {
        let columns = get_table_columns(path, table).await?;
        return Ok(TableData {
            columns: columns.into_iter().map(|c| c.name).collect(),
            rows: Vec::new(),
        });
    }

    let columns: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();
    let mut data_rows = Vec::new();

    for row in rows {
        let mut data_row = Vec::new();
        for i in 0..row.columns().len() {
            let col = &row.columns()[i];
            let type_name = col.type_info().name().to_uppercase();

            // Handle null values
            let raw_value = row.try_get_raw(i)?;
            if raw_value.is_null() {
                data_row.push(serde_json::Value::Null);
                continue;
            }

            let value = match type_name.as_str() {
                "INTEGER" | "INT" | "TINYINT" | "SMALLINT" | "MEDIUMINT" | "BIGINT"
                | "UNSIGNED BIG INT" | "INT2" | "INT8" => row
                    .try_get::<i64, _>(i)
                    .map(|v| serde_json::Value::Number(v.into()))
                    .unwrap_or(serde_json::Value::Null),
                "REAL" | "DOUBLE" | "DOUBLE PRECISION" | "FLOAT" | "NUMERIC" | "DECIMAL" => row
                    .try_get::<f64, _>(i)
                    .map(|v| {
                        serde_json::Number::from_f64(v)
                            .map(serde_json::Value::Number)
                            .unwrap_or_else(|| serde_json::Value::String(v.to_string()))
                    })
                    .unwrap_or(serde_json::Value::Null),
                "BOOLEAN" | "BOOL" => row
                    .try_get::<bool, _>(i)
                    .map(serde_json::Value::Bool)
                    .unwrap_or(serde_json::Value::Null),
                "BLOB" => row
                    .try_get::<Vec<u8>, _>(i)
                    .map(|v| serde_json::Value::String(format!("0x{}", hex::encode(v))))
                    .unwrap_or(serde_json::Value::Null),
                _ => {
                    // Fallback to string for everything else (TEXT, VARCHAR, etc.)
                    row.try_get::<String, _>(i)
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null)
                }
            };
            data_row.push(value);
        }
        data_rows.push(data_row);
    }

    Ok(TableData {
        columns,
        rows: data_rows,
    })
}

/// Gets schema information for a table.
pub async fn get_table_columns(path: &Path, table: &str) -> Result<Vec<ColumnInfo>> {
    let db_type = get_db_type(path).ok_or_else(|| anyhow::anyhow!("Unsupported database file"))?;

    if db_type != DatabaseType::SQLite {
        return Err(anyhow::anyhow!(
            "Schema inspection is only supported for SQLite databases."
        ));
    }

    if !table.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(anyhow::anyhow!("Invalid table name: {}", table));
    }

    let pool = get_connection(path).await?;
    let query_str = format!("PRAGMA table_info({})", table);
    let rows = sqlx::query(&query_str).fetch_all(&pool).await?;

    let columns = rows
        .iter()
        .map(|row| ColumnInfo {
            name: row.get(1),
            data_type: row.get(2),
            not_null: row.get::<i32, _>(3) != 0,
            default_value: row.get(4),
            primary_key: row.get::<i32, _>(5) != 0,
        })
        .collect();

    Ok(columns)
}

async fn get_connection(path: &Path) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::new().filename(path).read_only(true);

    SqlitePool::connect_with(options)
        .await
        .context(format!("Failed to connect to database at {:?}", path))
}
