use mc_server_wrapper_core::server::{ServerHandle, ServerStatus};
use mc_server_wrapper_core::config::ServerConfig;
use tempfile::tempdir;
use tokio::time::{sleep, Duration, timeout};
use std::fs;
use tracing::info;

#[tokio::test]
async fn test_stdio_redirection() {
    let dir = tempdir().unwrap();
    let working_dir = dir.path().to_path_buf();
    
    #[cfg(target_os = "windows")]
    let (script_name, script_content) = ("mock_server.bat", r#"@echo off
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
:loop
set /p cmd=
if "%cmd%"=="stop" (
    echo [Server thread/INFO]: Stopping server
    exit /b 0
)
echo Received: %cmd%
goto loop
"#);
    
    #[cfg(not(target_os = "windows"))]
    let (script_name, script_content) = ("mock_server.sh", r#"#!/bin/sh
echo '[Server thread/INFO]: Done (1.23s)! For help, type "help"'
while read cmd; do
    if [ "$cmd" = "stop" ]; then
        echo '[Server thread/INFO]: Stopping server'
        exit 0
    fi
    echo "Received: $cmd"
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
        name: "Test Server Stdio".to_string(),
        run_script: Some(script_path.to_string_lossy().to_string()),
        working_dir: working_dir.clone(),
        ..Default::default()
    };

    let handle = ServerHandle::new(config);
    let mut log_rx = handle.subscribe_logs();

    handle.start().await.expect("Failed to start server");

    // Wait for "Done" log
    let mut found_done = false;
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(5) {
        if let Ok(Ok(line)) = timeout(Duration::from_millis(100), log_rx.recv()).await {
            if line.contains("Done") {
                found_done = true;
                break;
            }
        }
    }
    assert!(found_done, "Server did not emit Done log");
    
    // Give it a moment to update status
    sleep(Duration::from_millis(200)).await;
    assert_eq!(handle.get_status().await, ServerStatus::Running);

    // Test sending command
    handle.send_command("hello").await.expect("Failed to send command");

    let mut found_received = false;
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(5) {
        if let Ok(Ok(line)) = timeout(Duration::from_millis(100), log_rx.recv()).await {
            if line.contains("Received: hello") {
                found_received = true;
                break;
            }
        }
    }
    assert!(found_received, "Server did not receive command");

    handle.stop().await.expect("Failed to stop server");
    
    // Wait for stop
    let mut stopped = false;
    for _ in 0..50 {
        if handle.get_status().await == ServerStatus::Stopped {
            stopped = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(stopped, "Server did not stop");
}

#[tokio::test]
async fn test_graceful_termination() {
    let dir = tempdir().unwrap();
    let working_dir = dir.path().to_path_buf();
    
    #[cfg(target_os = "windows")]
    let (script_name, script_content) = ("mock_graceful.bat", r#"@echo off
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
:loop
set /p cmd=
if "%cmd%"=="stop" (
    echo [Server thread/INFO]: Stopping server
    timeout /t 1 /nobreak > nul
    exit /b 0
)
goto loop
"#);
    
    #[cfg(not(target_os = "windows"))]
    let (script_name, script_content) = ("mock_graceful.sh", r#"#!/bin/sh
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
        name: "Test Server Graceful".to_string(),
        run_script: Some(script_path.to_string_lossy().to_string()),
        working_dir: working_dir.clone(),
        ..Default::default()
    };

    let handle = ServerHandle::new(config);
    handle.start().await.expect("Failed to start server");

    // Wait for running
    for _ in 0..50 {
        if handle.get_status().await == ServerStatus::Running {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    let start_stop = std::time::Instant::now();
    handle.stop().await.expect("Failed to stop server");

    // Wait for stopped
    let mut stopped = false;
    for _ in 0..50 {
        if handle.get_status().await == ServerStatus::Stopped {
            stopped = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(stopped, "Server did not stop gracefully");
    assert!(start_stop.elapsed() < Duration::from_secs(10), "Stop took too long");
}

#[tokio::test]
async fn test_forced_termination() {
    let _ = timeout(Duration::from_secs(30), async {
        let dir = tempdir().unwrap();
        let working_dir = dir.path().to_path_buf();
        
        // Script that ignores input and just hangs
        #[cfg(target_os = "windows")]
        let (script_name, script_content) = ("mock_hang.bat", r#"@echo off
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
:loop
timeout /t 60 /nobreak > nul
goto loop
"#);
        
        #[cfg(not(target_os = "windows"))]
        let (script_name, script_content) = ("mock_hang.sh", r#"#!/bin/sh
echo '[Server thread/INFO]: Done (1.23s)! For help, type "help"'
while true; do
    sleep 60
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
            name: "Test Server Hang".to_string(),
            run_script: Some(script_path.to_string_lossy().to_string()),
            working_dir: working_dir.clone(),
            stop_timeout: 2,
            ..Default::default()
        };

        let handle = ServerHandle::new(config);
        handle.start().await.expect("Failed to start server");

        // Wait for running
        for _ in 0..50 {
            if handle.get_status().await == ServerStatus::Running {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }

        // This will take a few seconds because of the reduced timeout in ops.rs during tests
        info!("Starting stop (waiting for forced termination)...");
        handle.stop().await.expect("Failed to stop server");

        // Wait for stopped
        let mut stopped = false;
        // We wait up to 10 seconds
        for _ in 0..100 {
            if handle.get_status().await == ServerStatus::Stopped {
                stopped = true;
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
        assert!(stopped, "Server was not killed after timeout");
    }).await.expect("Test timed out");
}

#[tokio::test]
async fn test_resource_usage() {
    let dir = tempdir().unwrap();
    let working_dir = dir.path().to_path_buf();
    
    // Script that does some "work" (looping) to use CPU
    #[cfg(target_os = "windows")]
    let (script_name, script_content) = ("mock_usage.bat", r#"@echo off
echo [Server thread/INFO]: Done (1.23s)! For help, type "help"
:loop
echo Working...
timeout /t 1 /nobreak > nul
goto loop
"#);
    
    #[cfg(not(target_os = "windows"))]
    let (script_name, script_content) = ("mock_usage.sh", r#"#!/bin/sh
echo '[Server thread/INFO]: Done (1.23s)! For help, type "help"'
while true; do
    echo "Working..."
    sleep 1
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
        name: "Test Server Usage".to_string(),
        run_script: Some(script_path.to_string_lossy().to_string()),
        working_dir: working_dir.clone(),
        ..Default::default()
    };

    let handle = ServerHandle::new(config);
    handle.start().await.expect("Failed to start server");

    // Wait for status to be Running
    for _ in 0..50 {
        if handle.get_status().await == ServerStatus::Running {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    // Wait for resource monitor to kick in (it sleeps for 2s in ops.rs)
    sleep(Duration::from_secs(5)).await;

    let usage = handle.get_usage().await;
    // We can't guarantee high CPU/Memory for a mock script, but it should be tracked
    // Memory usage should at least be greater than 0 for any running process
    assert!(usage.memory_usage > 0, "Memory usage should be greater than 0");
    
    handle.stop().await.expect("Failed to stop server");
}
