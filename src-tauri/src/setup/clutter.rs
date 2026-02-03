use tauri::App;
use std::path::Path;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub fn check_clutter(app: &mut App, exe_path: &Path) {
    let has_clutter = tauri::async_runtime::block_on(async {
        mc_server_wrapper_core::init::has_folder_clutter(exe_path).await.unwrap_or(false)
    });

    if has_clutter {
        let confirmed = tauri::async_runtime::block_on(async {
            app.dialog()
                .message("This folder contains other files and folders, we recommend putting the .exe executable in its OWN folder to avoid clutter and potential issues. Do you want to continue anyway?")
                .kind(MessageDialogKind::Warning)
                .title("Folder Clutter Detected")
                .buttons(MessageDialogButtons::YesNo)
                .blocking_show()
        });

        if !confirmed {
            app.handle().exit(0);
        }
    }
}
