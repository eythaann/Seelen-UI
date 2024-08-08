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

// ============== SETTINGS BY APP ==============

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum AppExtraFlag {
    Float,
    Force,
    Unmanage,
    Pinned,
    // only for backwards compatibility
    ObjectNameChange,
    Layered,
    BorderOverflow,
    TrayAndMultiWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AppIdentifierType {
    #[serde(alias = "exe")]
    Exe,
    #[serde(alias = "class")]
    Class,
    #[serde(alias = "title")]
    Title,
    #[serde(alias = "path")]
    Path,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MatchingStrategy {
    #[serde(alias = "equals")]
    Equals,
    #[serde(alias = "startsWith")]
    StartsWith,
    #[serde(alias = "endsWith")]
    EndsWith,
    #[serde(alias = "contains")]
    Contains,
    #[serde(alias = "regex")]
    Regex,
    // only for backwards compatibility
    #[serde(alias = "legacy")]
    Legacy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppIdentifier {
    pub id: String,
    pub kind: AppIdentifierType,
    pub matching_strategy: MatchingStrategy,
    #[serde(default)]
    pub negation: bool,
    #[serde(default)]
    pub and: Vec<AppIdentifier>,
    #[serde(default)]
    pub or: Vec<AppIdentifier>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AppConfig {
    pub name: String,
    pub category: Option<String>,
    pub bound_monitor_idx: Option<usize>,
    pub bound_workspace_name: Option<String>,
    pub identifier: AppIdentifier,
    #[serde(default)]
    pub options: Vec<AppExtraFlag>,
    #[serde(default)]
    pub is_bundled: bool,
}
