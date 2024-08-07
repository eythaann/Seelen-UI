use serde::{Deserialize, Serialize};

// ============== THEMES ==============

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeCss {
    pub weg: String,
    pub toolbar: String,
    pub wm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeInfo {
    pub display_name: String,
    pub author: String,
    pub description: String,
    pub filename: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Theme {
    pub info: ThemeInfo,
    pub styles: ThemeCss,
}

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
