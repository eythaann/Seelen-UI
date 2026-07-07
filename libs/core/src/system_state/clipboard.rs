use serde::Serialize;

/// Full clipboard state — emitted as a single payload on every relevant change.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct ClipboardData {
    /// Whether the user has enabled clipboard history in Windows Settings.
    pub is_history_enabled: bool,
    /// All clipboard history entries, newest first.
    pub history: Vec<ClipboardEntry>,
}

/// Content of a clipboard entry.
#[derive(Debug, Default, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ClipboardEntryContent {
    pub text: Option<String>,
    pub html: Option<String>,
    pub rtf: Option<String>,
    pub application_link: Option<String>,
    pub web_link: Option<String>,
    /// Base64-encoded WebP image.
    pub bitmap: Option<String>,
    pub files: Option<Vec<String>>,
}

/// A single entry in the Windows clipboard history.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ClipboardEntry {
    /// Unique identifier assigned by Windows.
    pub id: String,
    /// Unix epoch milliseconds (ms since 1970-01-01).
    pub timestamp: i64,
    /// Display name of the application that placed the entry on the clipboard.
    pub source_app_name: Option<String>,
    /// Base64-encoded WebP image of the app logo.
    pub source_app_logo: Option<String>,
    pub content: ClipboardEntryContent,
}
