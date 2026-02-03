use mc_server_wrapper_core::players::{self, PlayerEntry, OpEntry, BannedPlayerEntry};
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
