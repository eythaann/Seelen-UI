mod settings;
mod settings_by_app;
mod theme;

use serde::{Deserialize, Serialize};
pub use settings::*;
pub use settings_by_app::*;
pub use theme::*;

// ============== PLACEHOLDERS ==============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct PlaceholderInfo {
    pub display_name: String,
    pub author: String,
    pub description: String,
    pub filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Placeholder {
    pub info: PlaceholderInfo,
    pub left: Vec<serde_yaml::Value>,
    pub center: Vec<serde_yaml::Value>,
    pub right: Vec<serde_yaml::Value>,
}

// ============== WEG ==============

pub type WegItems = serde_yaml::Value;