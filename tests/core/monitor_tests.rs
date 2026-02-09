use mc_server_wrapper_core::server::ServerHandle;

#[test]
fn test_is_ready_line() {
    assert!(ServerHandle::is_ready_line(&"[16:46:29 INFO]: Listening on /0.0.0.0:25577".to_lowercase()));
    assert!(ServerHandle::is_ready_line(&"Done (1.23s)! For help, type \"help\"".to_lowercase()));
    assert!(ServerHandle::is_ready_line(&"Server started.".to_lowercase()));
    assert!(ServerHandle::is_ready_line(&"RCON running on 0.0.0.0:25575".to_lowercase()));
    
    assert!(!ServerHandle::is_ready_line(&"Loading libraries, please wait...".to_lowercase()));
    assert!(!ServerHandle::is_ready_line(&"Checking for updates...".to_lowercase()));
}
