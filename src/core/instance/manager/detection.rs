use std::path::Path;
use std::io::Read;
use regex::Regex;
use super::InstanceManager;

impl InstanceManager {
    pub(crate) async fn detect_minecraft_version(&self, instance_path: &Path, jar_name: &str) -> String {
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
                                if let Some(version) = json.get("depends").and_then(|d| d.get("minecraft")).and_then(|v| v.as_str()) {
                                    // Remove common version constraints like ">=1.20.1" or "~1.20"
                                    let cleaned = version.trim_start_matches(|c: char| !c.is_numeric());
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
                                if let Some(version) = json.get("quilt_loader").and_then(|l| l.get("depends")).and_then(|d| d.as_array()) {
                                    for dep in version {
                                        if dep.get("id").and_then(|i| i.as_str()) == Some("minecraft") {
                                            if let Some(v) = dep.get("versions").and_then(|v| v.as_str()).or_else(|| dep.get("version").and_then(|v| v.as_str())) {
                                                let cleaned = v.trim_start_matches(|c: char| !c.is_numeric());
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
            }).await.unwrap_or(None);

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

    pub(crate) fn parse_ram_from_script(&self, content: &str) -> Option<(u32, String, u32, String)> {
        let xms_regex = Regex::new(r"-Xms(\d+)([gGmMkK])").ok()?;
        let xmx_regex = Regex::new(r"-Xmx(\d+)([gGmMkK])").ok()?;

        let min = xms_regex.captures(content).and_then(|caps| {
            let val = caps[1].parse::<u32>().ok()?;
            let unit = caps[2].to_uppercase();
            Some((val, unit))
        });

        let max = xmx_regex.captures(content).and_then(|caps| {
            let val = caps[1].parse::<u32>().ok()?;
            let unit = caps[2].to_uppercase();
            Some((val, unit))
        });

        match (min, max) {
            (Some((min_val, min_unit)), Some((max_val, max_unit))) => {
                Some((min_val, min_unit, max_val, max_unit))
            }
            (None, Some((max_val, max_unit))) => {
                // If only Xmx is found, use a reasonable default for Xms (e.g., 512M or 1G)
                Some((1, "G".to_string(), max_val, max_unit))
            }
            (Some((min_val, min_unit)), None) => {
                // If only Xms is found, use it for Xmx too or a default
                Some((min_val, min_unit.clone(), min_val, min_unit))
            }
            _ => None,
        }
    }
}
