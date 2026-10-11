use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum SystemSettingKind {
    /// A page of the Windows Settings app (`ms-settings:` uri)
    Settings,
    /// A Control Panel applet (`shell:::{CLSID}`)
    ControlPanel,
}

/// Searchable entry of the system settings, collected from the Windows own indexes,
/// already localized to the user's display language.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct SystemSettingEntry {
    pub kind: SystemSettingKind,
    /// Localized display name
    pub title: String,
    /// Localized long description (tooltip)
    pub description: Option<String>,
    /// Localized search keywords/aliases
    pub keywords: Vec<String>,
    /// Uri or shell path to be opened via `OpenFile`
    pub target: String,
    /// Icon glyph character, always present in `font_family`
    pub glyph: Option<String>,
    /// Font family of the glyph
    pub font_family: Option<String>,
    /// Font file to be loaded via `@font-face`, only present when `font_family` is not
    /// installed on the system (private fonts of the Settings app).
    pub font_path: Option<PathBuf>,
}
