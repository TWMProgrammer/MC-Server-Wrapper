use super::InstanceManager;
use regex::Regex;
use std::io::Read;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct ParsedScriptInfo {
    pub min_ram: Option<u32>,
    pub min_ram_unit: Option<String>,
    pub max_ram: Option<u32>,
    pub max_ram_unit: Option<String>,
    pub jvm_args: Vec<String>,
    pub jar_name: Option<String>,
    pub server_args: Vec<String>,
    pub has_restart_loop: bool,
    pub java_path: Option<String>,
}

impl InstanceManager {
    pub(crate) async fn detect_minecraft_version(
        &self,
        instance_path: &Path,
        jar_name: &str,
    ) -> String {
        // 1. Try to read version.json from JAR (highest priority as it's definitive)
        let jar_path = instance_path.join(jar_name);
        if jar_path.exists() {
            let version = tokio::task::spawn_blocking({
                let jar_path = jar_path.to_path_buf();
                move || {
                    let file = std::fs::File::open(&jar_path).ok()?;
                    let mut archive = zip::ZipArchive::new(file).ok()?;

                    // Try version.json (Standard Vanilla/Paper)
                    if let Ok(mut version_file) = archive.by_name("version.json") {
                        let mut content = String::new();
                        if version_file.read_to_string(&mut content).is_ok() {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                                    return Some(id.to_string());
                                }
                            }
                        }
                    }

                    // Try fabric.mod.json (Fabric)
                    if let Ok(mut mod_file) = archive.by_name("fabric.mod.json") {
                        let mut content = String::new();
                        if mod_file.read_to_string(&mut content).is_ok() {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(version) = json
                                    .get("depends")
                                    .and_then(|d| d.get("minecraft"))
                                    .and_then(|v| v.as_str())
                                {
                                    // Remove common version constraints like ">=1.20.1" or "~1.20"
                                    let cleaned =
                                        version.trim_start_matches(|c: char| !c.is_numeric());
                                    return Some(cleaned.to_string());
                                }
                            }
                        }
                    }

                    // Try quilt.mod.json (Quilt)
                    if let Ok(mut mod_file) = archive.by_name("quilt.mod.json") {
                        let mut content = String::new();
                        if mod_file.read_to_string(&mut content).is_ok() {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(version) = json
                                    .get("quilt_loader")
                                    .and_then(|l| l.get("depends"))
                                    .and_then(|d| d.as_array())
                                {
                                    for dep in version {
                                        if dep.get("id").and_then(|i| i.as_str())
                                            == Some("minecraft")
                                        {
                                            if let Some(v) = dep
                                                .get("versions")
                                                .and_then(|v| v.as_str())
                                                .or_else(|| {
                                                    dep.get("version").and_then(|v| v.as_str())
                                                })
                                            {
                                                let cleaned =
                                                    v.trim_start_matches(|c: char| !c.is_numeric());
                                                return Some(cleaned.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    None
                }
            })
            .await
            .unwrap_or(None);

            if let Some(v) = version {
                return v;
            }
        }

        // 2. Try regex on filename
        // Matches things like 1.20.1, 1.8, etc.
        let version_regex = Regex::new(r"(1\.\d+(?:\.\d+)?)").unwrap();
        if let Some(caps) = version_regex.captures(jar_name) {
            return caps[1].to_string();
        }

        "Imported".to_string()
    }

    pub fn parse_script_content(content: &str) -> ParsedScriptInfo {
        let mut info = ParsedScriptInfo::default();

        // 1. Detect restart loop (common in batch/sh scripts)
        // Look for labels like :start and goto :start, or while loops
        let has_label = content.contains(":start") || content.contains(":loop");
        let has_goto = content.contains("goto start")
            || content.contains("goto :start")
            || content.contains("goto loop")
            || content.contains("goto :loop");
        let has_while = content.contains("while true") || content.contains("while [ 1 ]");
        info.has_restart_loop = (has_label && has_goto) || has_while;

        // 2. Find the java command line
        // We look for a line that contains 'java' and '-jar'
        // Also support variables like %JAVA% or "path/to/java"
        let java_line = content.lines().map(|l| l.trim()).find(|l| {
            let lower = l.to_lowercase();
            (lower.starts_with("java")
                || lower.contains(" java ")
                || lower.contains(".exe")
                || lower.contains("bin/java"))
                && lower.contains("-jar")
        });

        if let Some(line) = java_line {
            // Try to find where java ends and arguments begin
            // Usually it's the first space that isn't inside quotes
            let mut java_cmd = String::new();
            let mut remaining = "";
            let mut in_quotes = false;
            let mut found_space = false;

            for (i, c) in line.char_indices() {
                if c == '"' {
                    in_quotes = !in_quotes;
                } else if c == ' ' && !in_quotes {
                    java_cmd = line[..i].to_string();
                    remaining = &line[i..].trim();
                    found_space = true;
                    break;
                }
            }

            if !found_space {
                java_cmd = line.to_string();
                remaining = "";
            }

            // If java_cmd is not just "java", it might be a custom path
            let java_cmd_clean = java_cmd.trim_matches('"').to_string();
            if java_cmd_clean.to_lowercase() != "java" && !java_cmd_clean.is_empty() {
                info.java_path = Some(java_cmd_clean);
            }

            // Split by -jar to get JVM args and the rest
            if let Some(jar_idx) = remaining.find("-jar") {
                let jvm_part = &remaining[..jar_idx].trim();
                let rest = &remaining[jar_idx + 4..].trim();

                // Parse JVM args using a more robust splitter that respects quotes
                let mut args = Vec::new();
                let mut current_arg = String::new();
                let mut in_quotes = false;
                for c in jvm_part.chars() {
                    if c == '"' {
                        in_quotes = !in_quotes;
                    } else if c == ' ' && !in_quotes {
                        if !current_arg.is_empty() {
                            args.push(current_arg.clone());
                            current_arg.clear();
                        }
                    } else {
                        current_arg.push(c);
                    }
                }
                if !current_arg.is_empty() {
                    args.push(current_arg);
                }

                let xms_regex = Regex::new(r"-Xms(\d+)([gGmMkK])").unwrap();
                let xmx_regex = Regex::new(r"-Xmx(\d+)([gGmMkK])").unwrap();

                for arg in args {
                    if let Some(caps) = xms_regex.captures(&arg) {
                        info.min_ram = caps[1].parse::<u32>().ok();
                        info.min_ram_unit = Some(caps[2].to_uppercase());
                    } else if let Some(caps) = xmx_regex.captures(&arg) {
                        info.max_ram = caps[1].parse::<u32>().ok();
                        info.max_ram_unit = Some(caps[2].to_uppercase());
                    } else if !arg.is_empty() {
                        info.jvm_args.push(arg);
                    }
                }

                // Parse JAR name and server args
                let mut rest_args = Vec::new();
                let mut current_arg = String::new();
                let mut in_quotes = false;
                for c in rest.chars() {
                    if c == '"' {
                        in_quotes = !in_quotes;
                    } else if c == ' ' && !in_quotes {
                        if !current_arg.is_empty() {
                            rest_args.push(current_arg.clone());
                            current_arg.clear();
                        }
                    } else {
                        current_arg.push(c);
                    }
                }
                if !current_arg.is_empty() {
                    rest_args.push(current_arg);
                }

                if !rest_args.is_empty() {
                    info.jar_name = Some(rest_args[0].clone());
                    info.server_args = rest_args[1..].to_vec();
                }
            }
        }

        info
    }
}

#[cfg(test)]
mod tests {
    use crate::database::Database;
    use crate::instance::InstanceManager;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_parse_script_content_aikar() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Arc::new(Database::new(db_path).await.unwrap());
        let manager = InstanceManager::new(dir.path(), db).await.unwrap();

        let content = r#" @echo off 
  
 :start 
 java -Xms6144M -Xmx6144M -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+ParallelRefProcEnabled -XX:+PerfDisableSharedMem -XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1HeapRegionSize=8M -XX:G1HeapWastePercent=5 -XX:G1MaxNewSizePercent=40 -XX:G1MixedGCCountTarget=4 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1NewSizePercent=30 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:G1ReservePercent=20 -XX:InitiatingHeapOccupancyPercent=15 -XX:MaxGCPauseMillis=200 -XX:MaxTenuringThreshold=1 -XX:SurvivorRatio=32 -Dusing.aikars.flags=https://mcflags.emc.gs/ -Daikars.new.flags=true -jar minecraft-papermc-server.jar nogui 
  
 echo Server restarting... 
 echo Press CTRL + C to stop. 
 goto :start"#;

        let info = InstanceManager::parse_script_content(content);

        assert_eq!(info.min_ram, Some(6144));
        assert_eq!(info.min_ram_unit, Some("M".to_string()));
        assert_eq!(info.max_ram, Some(6144));
        assert_eq!(info.max_ram_unit, Some("M".to_string()));
        assert!(info.jvm_args.contains(&"-XX:+AlwaysPreTouch".to_string()));
        assert!(
            info.jvm_args
                .contains(&"-Daikars.new.flags=true".to_string())
        );
        assert_eq!(
            info.jar_name,
            Some("minecraft-papermc-server.jar".to_string())
        );
        assert_eq!(info.server_args, vec!["nogui".to_string()]);
        assert!(info.has_restart_loop);
    }

    #[tokio::test]
    async fn test_parse_script_custom_java() {
        let content = r#"@echo off
"C:\Program Files\Java\jdk-17\bin\java.exe" -Xms2G -Xmx4G -jar server.jar --nogui"#;

        let info = InstanceManager::parse_script_content(content);

        assert_eq!(info.min_ram, Some(2));
        assert_eq!(info.min_ram_unit, Some("G".to_string()));
        assert_eq!(info.max_ram, Some(4));
        assert_eq!(info.max_ram_unit, Some("G".to_string()));
        assert_eq!(
            info.java_path,
            Some(r#"C:\Program Files\Java\jdk-17\bin\java.exe"#.to_string())
        );
        assert_eq!(info.jar_name, Some("server.jar".to_string()));
        assert_eq!(info.server_args, vec!["--nogui".to_string()]);
    }
}
