use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::players;
use tauri::State;
use std::sync::Arc;
use uuid::Uuid;
use chrono;

#[tauri::command]
pub async fn open_player_list_file(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    list_type: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(instance) = instance_manager.get_instance(id).await.map_err(|e: anyhow::Error| e.to_string())? {
        let file_name = match list_type.as_str() {
            "whitelist" => "whitelist.json",
            "ops" => "ops.json",
            "banned-players" => "banned-players.json",
            "banned-ips" => "banned-ips.json",
            _ => return Err("Invalid list type".to_string()),
        };
        let file_path = instance.path.join(file_name);
        
        // Create the file if it doesn't exist, so the editor can open it
        if !file_path.exists() {
            tokio::fs::write(&file_path, "[]").await.map_err(|e| e.to_string())?;
        }

        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("powershell")
                .arg("-Command")
                .arg(format!("Start-Process '{}'", file_path.to_string_lossy()))
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(file_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(file_path)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    } else {
        Err("Instance not found".to_string())
    }
}

#[tauri::command]
pub async fn get_online_players(
    server_manager: State<'_, Arc<ServerManager>>,
    instance_id: String,
) -> Result<Vec<String>, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e: uuid::Error| e.to_string())?;
    if let Some(server) = server_manager.get_server(id).await {
        Ok(server.get_online_players().await)
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn get_players(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
) -> Result<players::AllPlayerLists, String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let whitelist = players::read_whitelist(&instance.path).await.map_err(|e| e.to_string())?;
    let ops = players::read_ops(&instance.path).await.map_err(|e| e.to_string())?;
    let banned_players = players::read_banned_players(&instance.path).await.map_err(|e| e.to_string())?;
    let banned_ips = players::read_banned_ips(&instance.path).await.map_err(|e| e.to_string())?;
    let user_cache = players::read_usercache(&instance.path).await.map_err(|e| e.to_string())?;
    
    Ok(players::AllPlayerLists {
        whitelist,
        ops,
        banned_players,
        banned_ips,
        user_cache,
    })
}

#[tauri::command]
pub async fn add_player(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    list_type: String,
    username: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    let (uuid, name) = players::fetch_player_uuid(&username).await.map_err(|e| e.to_string())?;

    match list_type.as_str() {
        "whitelist" => {
            let mut list = players::read_whitelist(&instance.path).await.map_err(|e| e.to_string())?;
            if !list.iter().any(|p| p.uuid == uuid) {
                list.push(players::PlayerEntry { uuid, name });
                players::write_whitelist(&instance.path, &list).await.map_err(|e| e.to_string())?;
            }
        },
        "ops" => {
            let mut list = players::read_ops(&instance.path).await.map_err(|e| e.to_string())?;
            if !list.iter().any(|p| p.uuid == uuid) {
                list.push(players::OpEntry { uuid, name, level: 4, bypasses_player_limit: false });
                players::write_ops(&instance.path, &list).await.map_err(|e| e.to_string())?;
            }
        },
        "banned-players" => {
            let mut list = players::read_banned_players(&instance.path).await.map_err(|e| e.to_string())?;
            if !list.iter().any(|p| p.uuid == uuid) {
                list.push(players::BannedPlayerEntry {
                    uuid,
                    name,
                    created: chrono::Utc::now().to_rfc3339(),
                    source: "Server Wrapper".to_string(),
                    expires: "forever".to_string(),
                    reason: "Banned by admin".to_string(),
                });
                players::write_banned_players(&instance.path, &list).await.map_err(|e| e.to_string())?;
            }
        },
        _ => return Err("Invalid list type".to_string()),
    }
    Ok(())
}

#[tauri::command]
pub async fn add_banned_ip(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    ip: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    let mut list = players::read_banned_ips(&instance.path).await.map_err(|e| e.to_string())?;
    if !list.iter().any(|p| p.ip == ip) {
        list.push(players::BannedIpEntry {
            ip,
            created: chrono::Utc::now().to_rfc3339(),
            source: "Server Wrapper".to_string(),
            expires: "forever".to_string(),
            reason: "Banned by admin".to_string(),
        });
        players::write_banned_ips(&instance.path, &list).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_player(
    instance_manager: State<'_, Arc<InstanceManager>>,
    instance_id: String,
    list_type: String,
    identifier: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&instance_id).map_err(|e| e.to_string())?;
    let instance = instance_manager.get_instance(id).await.map_err(|e| e.to_string())?
        .ok_or_else(|| "Instance not found".to_string())?;

    match list_type.as_str() {
        "whitelist" => {
            let mut list = players::read_whitelist(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.uuid != identifier);
            players::write_whitelist(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        "ops" => {
            let mut list = players::read_ops(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.uuid != identifier);
            players::write_ops(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        "banned-players" => {
            let mut list = players::read_banned_players(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.uuid != identifier);
            players::write_banned_players(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        "banned-ips" => {
            let mut list = players::read_banned_ips(&instance.path).await.map_err(|e| e.to_string())?;
            list.retain(|p| p.ip != identifier);
            players::write_banned_ips(&instance.path, &list).await.map_err(|e| e.to_string())?;
        },
        _ => return Err("Invalid list type".to_string()),
    }
    Ok(())
}
