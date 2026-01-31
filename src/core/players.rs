use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Result, Context};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerEntry {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpEntry {
    pub uuid: String,
    pub name: String,
    pub level: i32,
    #[serde(rename = "bypassesPlayerLimit")]
    pub bypasses_player_limit: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BannedPlayerEntry {
    pub uuid: String,
    pub name: String,
    pub created: String,
    pub source: String,
    pub expires: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BannedIpEntry {
    pub ip: String,
    pub created: String,
    pub source: String,
    pub expires: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllPlayerLists {
    pub whitelist: Vec<PlayerEntry>,
    pub ops: Vec<OpEntry>,
    pub banned_players: Vec<BannedPlayerEntry>,
    pub banned_ips: Vec<BannedIpEntry>,
}

pub async fn read_whitelist(path: &Path) -> Result<Vec<PlayerEntry>> {
    let file_path = path.join("whitelist.json");
    if !file_path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&file_path).await?;
    serde_json::from_str(&content).context("Failed to parse whitelist.json")
}

pub async fn write_whitelist(path: &Path, players: &[PlayerEntry]) -> Result<()> {
    let file_path = path.join("whitelist.json");
    let content = serde_json::to_string_pretty(players)?;
    fs::write(file_path, content).await.context("Failed to write whitelist.json")
}

pub async fn read_ops(path: &Path) -> Result<Vec<OpEntry>> {
    let file_path = path.join("ops.json");
    if !file_path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&file_path).await?;
    serde_json::from_str(&content).context("Failed to parse ops.json")
}

pub async fn write_ops(path: &Path, ops: &[OpEntry]) -> Result<()> {
    let file_path = path.join("ops.json");
    let content = serde_json::to_string_pretty(ops)?;
    fs::write(file_path, content).await.context("Failed to write ops.json")
}

pub async fn read_banned_players(path: &Path) -> Result<Vec<BannedPlayerEntry>> {
    let file_path = path.join("banned-players.json");
    if !file_path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&file_path).await?;
    serde_json::from_str(&content).context("Failed to parse banned-players.json")
}

pub async fn write_banned_players(path: &Path, banned: &[BannedPlayerEntry]) -> Result<()> {
    let file_path = path.join("banned-players.json");
    let content = serde_json::to_string_pretty(banned)?;
    fs::write(file_path, content).await.context("Failed to write banned-players.json")
}

pub async fn read_banned_ips(path: &Path) -> Result<Vec<BannedIpEntry>> {
    let file_path = path.join("banned-ips.json");
    if !file_path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&file_path).await?;
    serde_json::from_str(&content).context("Failed to parse banned-ips.json")
}

pub async fn write_banned_ips(path: &Path, banned: &[BannedIpEntry]) -> Result<()> {
    let file_path = path.join("banned-ips.json");
    let content = serde_json::to_string_pretty(banned)?;
    fs::write(file_path, content).await.context("Failed to write banned-ips.json")
}

#[derive(Debug, Deserialize)]
struct MojangProfile {
    id: String,
    name: String,
}

pub async fn fetch_player_uuid(username: &str) -> Result<(String, String)> {
    let client = reqwest::Client::new();
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", username);
    let resp = client.get(url).send().await?;
    
    if resp.status() == 404 {
        return Err(anyhow::anyhow!("Player not found"));
    }
    
    let profile: MojangProfile = resp.json().await?;
    
    // Mojang returns UUID without dashes, but Minecraft uses dashes in some files.
    // However, whitelist.json/ops.json usually work with both or prefer dashes.
    // Let's format it with dashes: 8-4-4-4-12
    let mut uuid = profile.id;
    if uuid.len() == 32 {
        uuid.insert(20, '-');
        uuid.insert(16, '-');
        uuid.insert(12, '-');
        uuid.insert(8, '-');
    }
    
    Ok((uuid, profile.name))
}
