use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeCss {
    /// Css Styles for the dock/taskbar
    pub weg: String,
    /// Css Styles for the window manager
    pub toolbar: String,
    /// Css Styles for the window manager
    pub wm: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeInfo {
    /// Display name of the theme
    pub display_name: String,
    /// Author of the theme
    pub author: String,
    /// Description of the theme
    pub description: String,
    /// Filename of the theme, is overridden by the program on load.
    pub filename: String,
    /// Tags to be used in search
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct Theme {
    /// Metadata about the theme
    pub info: ThemeInfo,
    /// Css Styles of the theme
    pub styles: ThemeCss,
}
