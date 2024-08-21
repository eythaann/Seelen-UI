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
pub struct TemporalPinnedWegItem {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeparatorWegItem {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MediaWegItem {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StartMenuWegItem {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum WegItem {
    PinnedApp(PinnedWegItem),
    TemporalPin(TemporalPinnedWegItem),
    Separator(SeparatorWegItem),
    Media(MediaWegItem),
    StartMenu(StartMenuWegItem),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct WegItems {
    left: Vec<WegItem>,
    center: Vec<WegItem>,
    right: Vec<WegItem>,
}
