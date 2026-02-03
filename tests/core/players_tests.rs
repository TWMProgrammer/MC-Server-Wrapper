use mc_server_wrapper_core::players::{self, PlayerEntry, OpEntry, BannedPlayerEntry, BannedIpEntry, UserCacheEntry};
use tempfile::tempdir;
use anyhow::Result;

#[tokio::test]
async fn test_whitelist_roundtrip() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let players = vec![
        PlayerEntry {
            uuid: "123-456".to_string(),
            name: "TestPlayer".to_string(),
        }
    ];
    
    players::write_whitelist(path, &players).await?;
    let read = players::read_whitelist(path).await?;
    
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].name, "TestPlayer");
    assert_eq!(read[0].uuid, "123-456");
    
    Ok(())
}

#[tokio::test]
async fn test_ops_roundtrip() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let ops = vec![
        OpEntry {
            uuid: "789-012".to_string(),
            name: "Admin".to_string(),
            level: 4,
            bypasses_player_limit: true,
        }
    ];
    
    players::write_ops(path, &ops).await?;
    let read = players::read_ops(path).await?;
    
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].name, "Admin");
    assert_eq!(read[0].level, 4);
    assert!(read[0].bypasses_player_limit);
    
    Ok(())
}

#[tokio::test]
async fn test_banned_players_roundtrip() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let banned = vec![
        BannedPlayerEntry {
            uuid: "bad-uuid".to_string(),
            name: "Griefer".to_string(),
            created: "2023-01-01".to_string(),
            source: "Console".to_string(),
            expires: "forever".to_string(),
            reason: "Griefing".to_string(),
        }
    ];
    
    players::write_banned_players(path, &banned).await?;
    let read = players::read_banned_players(path).await?;
    
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].name, "Griefer");
    assert_eq!(read[0].reason, "Griefing");
    
    Ok(())
}

#[tokio::test]
async fn test_banned_ips_roundtrip() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let banned = vec![
        BannedIpEntry {
            ip: "1.2.3.4".to_string(),
            created: "2023-01-01".to_string(),
            source: "Console".to_string(),
            expires: "forever".to_string(),
            reason: "Bad IP".to_string(),
        }
    ];
    
    players::write_banned_ips(path, &banned).await?;
    let read = players::read_banned_ips(path).await?;
    
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].ip, "1.2.3.4");
    assert_eq!(read[0].reason, "Bad IP");
    
    Ok(())
}

#[tokio::test]
async fn test_usercache_read() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let cache = vec![
        UserCacheEntry {
            uuid: "uuid1".to_string(),
            name: "Player1".to_string(),
            expires_on: "2023-12-31".to_string(),
        }
    ];
    
    let content = serde_json::to_string(&cache)?;
    tokio::fs::write(path.join("usercache.json"), content).await?;
    
    let read = players::read_usercache(path).await?;
    
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].name, "Player1");
    
    Ok(())
}

#[tokio::test]
async fn test_legacy_whitelist() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let content = "Player1\nPlayer2\n# Comment\n  Player3  ";
    tokio::fs::write(path.join("white-list.txt"), content).await?;
    
    let read = players::read_whitelist(path).await?;
    
    assert_eq!(read.len(), 3);
    assert_eq!(read[0].name, "Player1");
    assert_eq!(read[1].name, "Player2");
    assert_eq!(read[2].name, "Player3");
    assert_eq!(read[0].uuid, "");
    
    Ok(())
}

#[tokio::test]
async fn test_legacy_ops() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let content = "Admin1\nAdmin2";
    tokio::fs::write(path.join("ops.txt"), content).await?;
    
    let read = players::read_ops(path).await?;
    
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].name, "Admin1");
    assert_eq!(read[0].level, 4);
    
    Ok(())
}

#[tokio::test]
async fn test_legacy_banned_players() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let content = "BadGuy1\nBadGuy2|2023-01-01|Console|never|Reason";
    tokio::fs::write(path.join("banned-players.txt"), content).await?;
    
    let read = players::read_banned_players(path).await?;
    
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].name, "BadGuy1");
    assert_eq!(read[1].name, "BadGuy2");
    assert_eq!(read[1].reason, "Reason");
    
    Ok(())
}

#[tokio::test]
async fn test_legacy_banned_ips() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();
    
    let content = "1.2.3.4\n5.6.7.8|2023-01-01|Console|never|IP Reason";
    tokio::fs::write(path.join("banned-ips.txt"), content).await?;
    
    let read = players::read_banned_ips(path).await?;
    
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].ip, "1.2.3.4");
    assert_eq!(read[1].ip, "5.6.7.8");
    assert_eq!(read[1].reason, "IP Reason");
    
    Ok(())
}
