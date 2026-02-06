use std::path::{Path, PathBuf};
use tokio::fs;
use anyhow::{Result, Context};
use tracing::info;
use std::collections::HashSet;

/// Application directory structure
#[derive(Debug, Clone)]
pub struct AppDirs {
    pub backups: PathBuf,
    pub resources: PathBuf,
    pub server: PathBuf,
    pub cache: PathBuf,
    pub assets: PathBuf,
}

/// Checks if the given directory contains unrelated files or folders.
/// 
/// This logic ignores common development artifacts and files/folders
/// created by the application itself.
pub async fn has_folder_clutter(exe_dir: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(exe_dir).await?;
    
    // Items created by the app
    let app_items: HashSet<&str> = [
        "backups", "resources", "server", "java", "logs", "app_settings.json", "cache"
    ].into_iter().collect();

    // Development environment items
    let dev_folders: HashSet<&str> = [
        ".fingerprint", "deps", "examples", "incremental", "build", "target",
        "src", "src-tauri", "ui", "node_modules", ".github", "documents", ".git", "dist"
    ].into_iter().collect();

    let dev_files: HashSet<&str> = [
        ".cargo-lock", "Cargo.toml", "Cargo.lock", "package.json", "package-lock.json",
        "tsconfig.json", "vite.config.ts", "tailwind.config.js", "postcss.config.js",
        "index.html", ".gitignore", "README.md", "AGENTS.md"
    ].into_iter().collect();

    let dev_extensions: HashSet<&str> = [
        "pdb", "d", "dll", "lib", "rlib", "exp"
    ].into_iter().collect();

    let current_exe = std::env::current_exe().ok();
    let current_exe_name = current_exe
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str());

    let mut clutter_count = 0;

    while let Some(entry) = entries.next_entry().await? {
        let file_name_os = entry.file_name();
        let file_name = match file_name_os.to_str() {
            Some(name) => name,
            None => continue, // Skip files with invalid UTF-8 names
        };

        // 1. Ignore currently running executable
        if Some(file_name) == current_exe_name {
            continue;
        }

        // 2. Ignore application-created items
        if app_items.contains(file_name) {
            continue;
        }

        // 3. Ignore development items
        if dev_folders.contains(file_name) || dev_files.contains(file_name) {
            continue;
        }

        // 4. Ignore development extensions
        if let Some(ext) = Path::new(file_name).extension().and_then(|s| s.to_str()) {
            if dev_extensions.contains(ext) {
                continue;
            }
        }

        // 5. Ignore any .exe files (as per plan: "Executables: Any .exe file")
        if file_name.to_lowercase().ends_with(".exe") {
            continue;
        }

        // If we reached here, it's clutter
        info!("Detected clutter: {}", file_name);
        clutter_count += 1;
    }

    Ok(clutter_count > 0)
}

/// Initializes the application directory structure next to the executable.
/// 
/// # Arguments
/// * `base_path` - The base path where the folders should be created (usually the exe directory).
pub async fn init_directories(base_path: &Path) -> Result<AppDirs> {
    let backups = base_path.join("backups");
    let resources = base_path.join("resources");
    let server = base_path.join("server");
    let cache = base_path.join("cache");
    let assets = cache.join("assets");

    let dirs = [
        (&backups, "backups"),
        (&resources, "resources"),
        (&server, "server"),
        (&cache, "cache"),
        (&assets, "assets"),
    ];

    for (path, name) in dirs {
        if !path.exists() {
            fs::create_dir_all(path)
                .await
                .with_context(|| format!("Failed to create {} directory at {:?}", name, path))?;
            info!("Created {} directory: {:?}", name, path);
        }
    }

    Ok(AppDirs {
        backups,
        resources,
        server,
        cache,
        assets,
    })
}
