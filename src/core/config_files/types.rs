#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigFile {
    pub name: String,
    pub path: String, // Relative to instance root
    pub format: ConfigFormat,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ConfigFormat {
    Properties,
    Yaml,
    Toml,
    Json,
}
