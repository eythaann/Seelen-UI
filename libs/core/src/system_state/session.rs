/// Seelen cloud session data exposed to the UI.
/// Tokens are intentionally excluded — the background manages them exclusively.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct SeelenSession {
    pub id: String,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub plan: String,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
}

/// Status of the cloud backup sync, exposed to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct BackupStatus {
    /// RFC-3339 timestamp of the last successful sync, or `None` if never synced.
    pub last_sync: Option<String>,
}
