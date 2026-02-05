use super::super::ServerManager;
use crate::config::ServerConfig;
use crate::instance::{InstanceMetadata, LaunchMethod};

impl ServerManager {
    pub(crate) async fn build_server_config(&self, instance: &InstanceMetadata) -> ServerConfig {
        let is_bedrock = instance
            .mod_loader
            .as_deref()
            .map(|l| l.to_lowercase() == "bedrock")
            .unwrap_or(false);
        let bedrock_exe = if cfg!(windows) {
            "bedrock_server.exe"
        } else {
            "bedrock_server"
        };
        let bedrock_path = instance.path.join(bedrock_exe);
        let jar_path = instance.path.join("server.jar");

        let mut final_jar_path = if is_bedrock {
            Some(bedrock_path)
        } else {
            Some(jar_path)
        };
        let mut final_run_script = None;
        let mut args = vec!["nogui".to_string()];

        let loader_lower = instance.mod_loader.as_deref().map(|l| l.to_lowercase());

        // Check for Fabric server
        if loader_lower.as_deref() == Some("fabric")
            && instance.path.join("fabric-server.jar").exists()
        {
            final_jar_path = Some(instance.path.join("fabric-server.jar"));
        }

        // Check for run scripts (modern Forge/NeoForge)
        let run_script_name = if cfg!(windows) { "run.bat" } else { "run.sh" };
        if instance.path.join(run_script_name).exists() {
            final_run_script = Some(run_script_name.to_string());
            final_jar_path = None;
        }

        // Apply custom launch settings if it's an imported instance or has custom settings
        match instance.settings.launch_method {
            LaunchMethod::BatFile => {
                if let Some(bat) = &instance.settings.bat_file {
                    final_run_script = Some(bat.clone());
                    final_jar_path = None;
                    args.clear();
                }
            }
            LaunchMethod::StartupLine => {
                let is_imported = instance.version == "Imported";
                let has_specialized = final_run_script.is_some()
                    || (loader_lower.as_deref() == Some("fabric")
                        && instance.path.join("fabric-server.jar").exists());

                if is_imported || !has_specialized {
                    if let Some(jar_idx) = instance.settings.startup_line.find("-jar ") {
                        let after_jar = &instance.settings.startup_line[jar_idx + 5..];
                        let mut parts = after_jar.split_whitespace();
                        if let Some(jar_name) = parts.next() {
                            final_jar_path = Some(instance.path.join(jar_name));
                        }
                        args = parts.map(|s| s.to_string()).collect();
                    }
                }
            }
        }

        // Special case for Proxies: they don't need nogui
        if let Some(loader) = &loader_lower {
            if loader == "velocity" || loader == "bungeecord" {
                args.clear();
            }
        }

        // Special case for Bedrock: it's not a jar and doesn't need nogui
        if is_bedrock {
            args.clear();
        }

        // Resolve Java path
        let mut java_path = None;
        if let Some(java_override) = &instance.settings.java_path_override {
            if !java_override.is_empty() && java_override != "java" {
                // Check if it's a managed version ID
                if let Ok(settings) = self.config_manager.load().await {
                    if let Some(managed) = settings
                        .managed_java_versions
                        .iter()
                        .find(|v| v.id == *java_override)
                    {
                        java_path = Some(managed.path.clone());
                    } else {
                        // Check if it's a valid path on disk
                        let path = std::path::Path::new(java_override);
                        if path.exists() {
                            java_path = Some(path.to_path_buf());
                        }
                    }
                }
            }
        }

        let min_ram_unit = match instance.settings.min_ram_unit.as_str() {
            "GB" => "G",
            "MB" => "M",
            u => u,
        };
        let max_ram_unit = match instance.settings.max_ram_unit.as_str() {
            "GB" => "G",
            "MB" => "M",
            u => u,
        };

        ServerConfig {
            name: instance.name.clone(),
            max_memory: format!("{}{}", instance.settings.max_ram, max_ram_unit),
            min_memory: format!("{}{}", instance.settings.min_ram, min_ram_unit),
            jar_path: final_jar_path,
            run_script: final_run_script,
            args,
            working_dir: instance.path.clone(),
            java_path,
            crash_handling: instance.settings.crash_handling.clone(),
            stop_timeout: 30,
        }
    }
}
