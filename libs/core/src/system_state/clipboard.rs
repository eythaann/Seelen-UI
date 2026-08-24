use serde::{Deserialize, Serialize};

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
    /// Ids of entries pinned by the user. Pinned entries are kept when
    /// clearing the history.
    pub pinned_ids: Vec<String>,
}

/// Content of a clipboard entry.
#[derive(Debug, Default, Clone, Hash, Serialize, Deserialize)]
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

impl ClipboardEntryContent {
    /// Rough serialized size, in bytes — used only to enforce storage limits.
    pub fn approx_size(&self) -> usize {
        self.text.as_ref().map_or(0, String::len)
            + self.html.as_ref().map_or(0, String::len)
            + self.rtf.as_ref().map_or(0, String::len)
            + self.bitmap.as_ref().map_or(0, String::len)
            + self.application_link.as_ref().map_or(0, String::len)
            + self.web_link.as_ref().map_or(0, String::len)
            + self
                .files
                .as_ref()
                .map_or(0, |f| f.iter().map(String::len).sum())
    }

    /// Cheap fingerprint used to deduplicate re-copies of identical content.
    /// Derived from all fields via `#[derive(Hash)]`, so a new field is
    /// automatically covered here without needing to be listed again.
    pub fn fingerprint(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// A single entry in our own clipboard history store.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ClipboardEntry {
    /// Unique identifier assigned by us.
    pub id: String,
    /// Unix epoch milliseconds (ms since 1970-01-01).
    pub timestamp: i64,
    /// Display name of the application that placed the entry on the clipboard.
    pub source_app_name: Option<String>,
    /// App user model id of the application that placed the entry on the clipboard.
    /// Can be used by the front-end to resolve an icon via the IconPack feature.
    pub source_app_umid: Option<String>,
    /// Executable path of the application that placed the entry on the clipboard.
    /// Can be used by the front-end to resolve an icon via the IconPack feature.
    pub source_app_path: Option<String>,
    pub content: ClipboardEntryContent,
}
