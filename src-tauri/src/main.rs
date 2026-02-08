// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    {
        set_app_id();
        // Run shortcut creation in a separate thread to avoid blocking startup
        std::thread::spawn(|| {
            if let Err(e) = create_shortcut() {
                eprintln!("Failed to create shortcut: {}", e);
            }
        });
    }

    if let Err(e) = app_lib::run() {
        eprintln!("Fatal error: {:?}", e);
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn create_shortcut() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();

    // We create a shortcut in the Start Menu programs folder.
    // This is required for Windows Toast Notifications to work in non-installed (portable) apps.
    // Use a cleaner PowerShell script without escaped quotes in the format string
    let script = format!(
        "$path = '{}'; $shortcutPath = \"$env:APPDATA\\Microsoft\\Windows\\Start Menu\\Programs\\MC Server Wrapper.lnk\"; if (-not (Test-Path $shortcutPath)) {{ $shell = New-Object -ComObject WScript.Shell; $shortcut = $shell.CreateShortcut($shortcutPath); $shortcut.TargetPath = $path; $shortcut.Save(); }}",
        exe_path_str
    );

    Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .status()?;

    Ok(())
}

#[cfg(windows)]
fn set_app_id() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "shell32")]
    extern "system" {
        fn SetCurrentProcessExplicitAppUserModelID(AppID: *const u16) -> i32;
    }

    let id: Vec<u16> = OsStr::new("com.mc-server-wrapper.app")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
    }
}
