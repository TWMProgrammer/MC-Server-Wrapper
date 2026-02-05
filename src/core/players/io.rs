use super::types::*;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

pub async fn read_usercache(path: &Path) -> Result<Vec<UserCacheEntry>> {
    let file_path = path.join("usercache.json");
    if !file_path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(&file_path).await?;
    serde_json::from_str(&content).context("Failed to parse usercache.json")
}

pub async fn read_whitelist(path: &Path) -> Result<Vec<PlayerEntry>> {
    let json_path = path.join("whitelist.json");
    if json_path.exists() {
        let content = fs::read_to_string(&json_path).await?;
        return serde_json::from_str(&content).context("Failed to parse whitelist.json");
    }

    // Try legacy format
    let txt_path = path.join("white-list.txt");
    if txt_path.exists() {
        let content = fs::read_to_string(&txt_path).await?;
        let players = content
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|name| PlayerEntry {
                uuid: "".to_string(), // Legacy format doesn't have UUIDs
                name: name.to_string(),
            })
            .collect();
        return Ok(players);
    }

    Ok(vec![])
}

pub async fn write_whitelist(path: &Path, players: &[PlayerEntry]) -> Result<()> {
    let file_path = path.join("whitelist.json");
    let content = serde_json::to_string_pretty(players)?;
    fs::write(file_path, content)
        .await
        .context("Failed to write whitelist.json")
}

pub async fn read_ops(path: &Path) -> Result<Vec<OpEntry>> {
    let json_path = path.join("ops.json");
    if json_path.exists() {
        let content = fs::read_to_string(&json_path).await?;
        return serde_json::from_str(&content).context("Failed to parse ops.json");
    }

    // Try legacy format
    let txt_path = path.join("ops.txt");
    if txt_path.exists() {
        let content = fs::read_to_string(&txt_path).await?;
        let ops = content
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|name| OpEntry {
                uuid: "".to_string(),
                name: name.to_string(),
                level: 4, // Default level
                bypasses_player_limit: false,
            })
            .collect();
        return Ok(ops);
    }

    Ok(vec![])
}

pub async fn write_ops(path: &Path, ops: &[OpEntry]) -> Result<()> {
    let file_path = path.join("ops.json");
    let content = serde_json::to_string_pretty(ops)?;
    fs::write(file_path, content)
        .await
        .context("Failed to write ops.json")
}

pub async fn read_banned_players(path: &Path) -> Result<Vec<BannedPlayerEntry>> {
    let json_path = path.join("banned-players.json");
    if json_path.exists() {
        let content = fs::read_to_string(&json_path).await?;
        return serde_json::from_str(&content).context("Failed to parse banned-players.json");
    }

    // Try legacy format
    let txt_path = path.join("banned-players.txt");
    if txt_path.exists() {
        let content = fs::read_to_string(&txt_path).await?;
        let banned = content
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    BannedPlayerEntry {
                        uuid: "".to_string(),
                        name: parts[0].to_string(),
                        created: parts[1].to_string(),
                        source: parts[2].to_string(),
                        expires: parts[3].to_string(),
                        reason: parts[4].to_string(),
                    }
                } else {
                    BannedPlayerEntry {
                        uuid: "".to_string(),
                        name: line.to_string(),
                        created: "".to_string(),
                        source: "".to_string(),
                        expires: "".to_string(),
                        reason: "".to_string(),
                    }
                }
            })
            .collect();
        return Ok(banned);
    }

    Ok(vec![])
}

pub async fn write_banned_players(path: &Path, banned: &[BannedPlayerEntry]) -> Result<()> {
    let file_path = path.join("banned-players.json");
    let content = serde_json::to_string_pretty(banned)?;
    fs::write(file_path, content)
        .await
        .context("Failed to write banned-players.json")
}

pub async fn read_banned_ips(path: &Path) -> Result<Vec<BannedIpEntry>> {
    let json_path = path.join("banned-ips.json");
    if json_path.exists() {
        let content = fs::read_to_string(&json_path).await?;
        return serde_json::from_str(&content).context("Failed to parse banned-ips.json");
    }

    // Try legacy format
    let txt_path = path.join("banned-ips.txt");
    if txt_path.exists() {
        let content = fs::read_to_string(&txt_path).await?;
        let banned = content
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .map(|line| {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 5 {
                    BannedIpEntry {
                        ip: parts[0].to_string(),
                        created: parts[1].to_string(),
                        source: parts[2].to_string(),
                        expires: parts[3].to_string(),
                        reason: parts[4].to_string(),
                    }
                } else {
                    BannedIpEntry {
                        ip: line.to_string(),
                        created: "".to_string(),
                        source: "".to_string(),
                        expires: "".to_string(),
                        reason: "".to_string(),
                    }
                }
            })
            .collect();
        return Ok(banned);
    }

    Ok(vec![])
}

pub async fn write_banned_ips(path: &Path, banned: &[BannedIpEntry]) -> Result<()> {
    let file_path = path.join("banned-ips.json");
    let content = serde_json::to_string_pretty(banned)?;
    fs::write(file_path, content)
        .await
        .context("Failed to write banned-ips.json")
}
