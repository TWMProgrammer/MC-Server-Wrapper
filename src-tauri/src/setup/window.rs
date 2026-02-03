use tauri::{App, Manager};

pub fn setup_window(app: &mut App) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let size = monitor.size();
            let width = (size.width as f64 * 0.7) as u32;
            let height = (size.height as f64 * 0.7) as u32;
            let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width,
                height,
            }));
            let _ = window.center();
        }
    }
}
