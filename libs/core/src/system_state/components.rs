use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Disk {
    pub name: String,
    pub file_system: String,
    pub total_space: u64,
    pub available_space: u64,
    pub mount_point: PathBuf,
    pub is_removable: bool,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatistics {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub total: u64,
    pub free: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct Core {
    pub name: String,
    pub brand: String,
    pub usage: f32,
    pub frequency: u64,
}
