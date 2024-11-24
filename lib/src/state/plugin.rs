use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub target: String,
    pub plugin: serde_yaml::Value,
    #[serde(default)]
    pub bundled: bool,
}