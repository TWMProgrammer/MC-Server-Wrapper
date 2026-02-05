use serde::{Deserialize, Serialize};

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
pub struct UserCacheEntry {
    pub uuid: String,
    pub name: String,
    #[serde(rename = "expiresOn")]
    pub expires_on: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllPlayerLists {
    pub whitelist: Vec<PlayerEntry>,
    pub ops: Vec<OpEntry>,
    pub banned_players: Vec<BannedPlayerEntry>,
    pub banned_ips: Vec<BannedIpEntry>,
    pub user_cache: Vec<UserCacheEntry>,
}
