use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModLoader {
    pub name: String,
    pub versions: Vec<String>,
}
