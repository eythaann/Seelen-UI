use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct TrashBinInfo {
    /// Number of items currently in the recycle bin
    pub item_count: i64,
    /// Total size of all items in bytes
    pub size_in_bytes: i64,
}
