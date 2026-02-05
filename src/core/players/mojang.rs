use serde::Deserialize;
use anyhow::{Result, anyhow};

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
        return Err(anyhow!("Player not found"));
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
