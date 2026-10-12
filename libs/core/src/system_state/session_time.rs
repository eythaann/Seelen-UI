use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct ActiveSessionTime {
    pub active_seconds: u64,
    pub longest_active_seconds: u64,
    pub is_idle: bool,
}
