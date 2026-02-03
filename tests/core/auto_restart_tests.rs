use mc_server_wrapper_core::server::{ServerHandle, ServerStatus};
use mc_server_wrapper_core::config::ServerConfig;
use mc_server_wrapper_core::instance::CrashHandlingMode;
use tempfile::tempdir;
use tokio::time::{sleep, Duration, timeout};
use std::fs;

#[tokio::test]
async fn test_auto_restart_on_crash() {
    let dir = tempdir().unwrap();
    let working_dir = dir.path().to_path_buf();
    
    #[cfg(target_os = "windows")]
    let (script_name, script_content) = ("crash_server.bat", r#"@echo off
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
exit /b 1
"#);
    
    #[cfg(not(target_os = "windows"))]
    let (script_name, script_content) = ("crash_server.sh", r#"#!/bin/sh
echo '[Server thread/INFO]: Done (1.23s)! For help, type "help"'
exit 1
"#);

    let script_path = working_dir.join(script_name);
    fs::write(&script_path, script_content).unwrap();
    
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let config = ServerConfig {
        name: "Crash Server".to_string(),
        run_script: Some(script_path.to_string_lossy().to_string()),
        working_dir: working_dir.clone(),
        crash_handling: CrashHandlingMode::Aggressive,
        ..Default::default()
    };

    let handle = ServerHandle::new(config);
    let mut log_rx = handle.subscribe_logs();

    handle.start().await.expect("Failed to start server");

    // 1. Wait for first start
    let mut starts = 0;
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(15) {
        if let Ok(Ok(line)) = timeout(Duration::from_millis(100), log_rx.recv()).await {
            if line.contains("Done") {
                starts += 1;
                if starts == 2 {
                    break;
                }
            }
        }
    }

    assert_eq!(starts, 2, "Server did not restart automatically");
    
    // Cleanup: Stop the server (it might be in Starting/Running state again)
    // Since our mock script exits immediately, it will keep restarting.
    // Let's disable auto-restart and wait for it to stop.
    let mut cfg = handle.get_config().await;
    cfg.crash_handling = CrashHandlingMode::Nothing;
    handle.update_config(cfg).await;
    
    // Wait for it to become Crashed or Stopped after the last restart
    let mut finished = false;
    for _ in 0..100 {
        let status = handle.get_status().await;
        if status == ServerStatus::Crashed || status == ServerStatus::Stopped {
            finished = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(finished, "Server did not finish after disabling auto-restart");
}

#[tokio::test]
async fn test_state_transitions() {
    let dir = tempdir().unwrap();
    let working_dir = dir.path().to_path_buf();
    
    #[cfg(target_os = "windows")]
    let (script_name, script_content) = ("state_server.bat", r#"@echo off
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
:loop
set /p cmd=
if "%cmd%"=="stop" (
    echo [Server thread/INFO]: Stopping server
    timeout /t 1 > nul
    exit /b 0
)
goto loop
"#);
    
    #[cfg(not(target_os = "windows"))]
    let (script_name, script_content) = ("state_server.sh", r#"#!/bin/sh
echo '[Server thread/INFO]: Done (1.23s)! For help, type "help"'
while read cmd; do
    if [ "$cmd" = "stop" ]; then
        echo '[Server thread/INFO]: Stopping server'
        sleep 1
        exit 0
    fi
done
"#);

    let script_path = working_dir.join(script_name);
    fs::write(&script_path, script_content).unwrap();
    
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let config = ServerConfig {
        name: "State Server".to_string(),
        run_script: Some(script_path.to_string_lossy().to_string()),
        working_dir: working_dir.clone(),
        ..Default::default()
    };

    let handle = ServerHandle::new(config);

    // Initial state: Stopped
    assert_eq!(handle.get_status().await, ServerStatus::Stopped);

    // Start: Starting
    handle.start().await.expect("Failed to start server");
    assert_eq!(handle.get_status().await, ServerStatus::Starting);

    // Wait for Running
    let mut running = false;
    for _ in 0..50 {
        if handle.get_status().await == ServerStatus::Running {
            running = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(running, "Server did not reach Running state");

    // Stop: Stopping
    let stop_handle = handle.clone();
    let stop_task = tokio::spawn(async move {
        stop_handle.stop().await.expect("Failed to stop server");
    });
    
    // Check for Stopping state (might be brief)
    let mut found_stopping = false;
    for _ in 0..10 {
        if handle.get_status().await == ServerStatus::Stopping {
            found_stopping = true;
            break;
        }
        sleep(Duration::from_millis(10)).await;
    }
    assert!(found_stopping, "Server did not reach Stopping state");

    stop_task.await.unwrap();

    // Wait for Stopped
    let mut stopped = false;
    for _ in 0..50 {
        if handle.get_status().await == ServerStatus::Stopped {
            stopped = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(stopped, "Server did not reach Stopped state");
}

#[tokio::test]
async fn test_manual_stop_command() {
    let dir = tempdir().unwrap();
    let working_dir = dir.path().to_path_buf();
    
    #[cfg(target_os = "windows")]
    let (script_name, script_content) = ("manual_stop.bat", r#"@echo off
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
set /p cmd=
if "%cmd%"=="stop" (
    exit /b 0
)
exit /b 1
"#);
    
    #[cfg(not(target_os = "windows"))]
    let (script_name, script_content) = ("manual_stop.sh", r#"#!/bin/sh
echo '[Server thread/INFO]: Done (1.23s)! For help, type "help"'
read cmd
if [ "$cmd" = "stop" ]; then
    exit 0
fi
exit 1
"#);

    let script_path = working_dir.join(script_name);
    fs::write(&script_path, script_content).unwrap();
    
    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let config = ServerConfig {
        name: "Manual Stop Server".to_string(),
        run_script: Some(script_path.to_string_lossy().to_string()),
        working_dir: working_dir.clone(),
        ..Default::default()
    };

    let handle = ServerHandle::new(config);
    handle.start().await.expect("Failed to start server");

    // Wait for Running
    let mut running = false;
    for _ in 0..50 {
        if handle.get_status().await == ServerStatus::Running {
            running = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(running, "Server did not reach Running state");

    // Send "stop" command directly
    handle.send_command("stop").await.expect("Failed to send command");

    // Wait for Stopped (should NOT be Crashed even though we didn't call handle.stop())
    let mut stopped = false;
    for _ in 0..50 {
        let status = handle.get_status().await;
        if status == ServerStatus::Stopped {
            stopped = true;
            break;
        }
        if status == ServerStatus::Crashed {
            panic!("Server detected as Crashed instead of Stopped");
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(stopped, "Server did not reach Stopped state after manual command");
}
