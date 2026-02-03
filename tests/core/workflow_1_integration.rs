use mc_server_wrapper_core::manager::ServerManager;
use mc_server_wrapper_core::instance::InstanceManager;
use mc_server_wrapper_core::app_config::GlobalConfigManager;
use mc_server_wrapper_core::server::{ServerStatus};
use tempfile::tempdir;
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration, timeout};
use std::fs;

#[tokio::test]
async fn test_workflow_1_fresh_installation() -> Result<()> {
    let dir = tempdir()?;
    let instances_dir = dir.path().join("instances");
    let config_dir = dir.path().join("config");
    
    fs::create_dir_all(&instances_dir)?;
    fs::create_dir_all(&config_dir)?;

    let instance_manager = InstanceManager::new(&instances_dir).await?;
    let config_manager = GlobalConfigManager::new(config_dir.join("config.json"));
    
    let manager = ServerManager::new(
        Arc::new(instance_manager),
        Arc::new(config_manager),
    );

    // 1. User creates a new instance (e.g., Paper 1.20.1).
    let instance = manager.get_instance_manager().create_instance("Fresh Paper", "1.20.1").await?;
    assert_eq!(instance.name, "Fresh Paper");
    assert_eq!(instance.version, "1.20.1");

    // Mocking the installation (normally this would be done by the UI/downloader)
    // We create a mock run script to simulate the server starting.
    #[cfg(target_os = "windows")]
    let (script_name, script_content) = ("run.bat", r#"@echo off
echo [Server thread/INFO]: Starting minecraft server version 1.20.1
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
:loop
set /p cmd=
if "%cmd%"=="stop" (
    echo [Server thread/INFO]: Stopping server
    exit /b 0
)
goto loop
"#);

    #[cfg(not(target_os = "windows"))]
    let (script_name, script_content) = ("run.sh", r#"#!/bin/sh
echo '[Server thread/INFO]: Starting minecraft server version 1.20.1'
echo '[Server thread/INFO]: Done (1.23s)! For help, type "help"'
while read cmd; do
    if [ "$cmd" = "stop" ]; then
        echo '[Server thread/INFO]: Stopping server'
        exit 0
    fi
done
"#);

    let script_path = instance.path.join(script_name);
    fs::write(&script_path, script_content)?;
    
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // 2. User starts the server and waits for "Done" in logs.
    let server = manager.get_or_create_server(instance.id).await?;
    let mut log_rx = server.subscribe_logs();
    
    server.start().await?;

    let mut found_done = false;
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(10) {
        if let Ok(Ok(line)) = timeout(Duration::from_millis(500), log_rx.recv()).await {
            if line.contains("Done") {
                found_done = true;
                break;
            }
        }
    }
    assert!(found_done, "Server did not emit Done log");
    
    // Give it a moment to update status
    sleep(Duration::from_millis(500)).await;
    assert_eq!(server.get_status().await, ServerStatus::Running);

    // 3. User stops the server.
    server.stop().await?;
    
    let mut stopped = false;
    for _ in 0..20 {
        if server.get_status().await == ServerStatus::Stopped {
            stopped = true;
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }
    assert!(stopped, "Server did not stop in time");

    Ok(())
}
