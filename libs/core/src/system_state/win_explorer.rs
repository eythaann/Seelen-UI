use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct StartMenuItem {
    pub path: PathBuf,
    pub umid: Option<String>,
    pub toast_activator: Option<String>,
    /// Will be present if the item is a shortcut
    pub target: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrayIcon {
    pub label: String,
    pub registry: RegistryNotifyIcon,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RegistryNotifyIcon {
    /// can be used as a unique identifier of the registered tray icon
    pub key: String,
    pub executable_path: PathBuf,
    pub initial_tooltip: Option<String>,
    /// PNG image of the cached icon
    pub icon_snapshot: Option<Vec<u8>>,
    pub icon_guid: Option<String>,
    pub icon_uid: Option<u32>,
    pub is_promoted: bool,
    pub is_running: bool,
}
