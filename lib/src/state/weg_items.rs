use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PinnedWegItem {
    /// executable path
    exe: String,
    /// command to open the app using explorer.exe (uwp apps starts with `shell:AppsFolder`)
    execution_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemporalPinnedWegItem {
    /// executable path
    exe: String,
    /// command to open the app using explorer.exe (uwp apps starts with `shell:AppsFolder`)
    execution_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum WegItem {
    PinnedApp(PinnedWegItem),
    TemporalPin(TemporalPinnedWegItem),
    Separator,
    Media,
    StartMenu,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct WegItems {
    left: Vec<WegItem>,
    center: Vec<WegItem>,
    right: Vec<WegItem>,
}

impl Default for WegItems {
    fn default() -> Self {
        Self {
            left: vec![WegItem::StartMenu],
            center: vec![WegItem::PinnedApp(PinnedWegItem {
                exe: "C:\\Windows\\explorer.exe".to_string(),
                execution_path: "C:\\Windows\\explorer.exe".to_string(),
            })],
            right: vec![WegItem::Media],
        }
    }
}
