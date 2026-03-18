use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct TrashBinInfo {
    /// Number of items currently in the recycle bin
    pub item_count: i64,
    /// Total size of all items in bytes
    pub size_in_bytes: i64,
}
