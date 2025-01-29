use std::path::PathBuf;

use serde::Serialize;
use windows::Win32::UI::Accessibility::IUIAutomationElement;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayIcon {
    #[serde(skip)]
    pub ui_automation: IUIAutomationElement,
    pub label: String,
    pub registry: RegistryNotifyIcon,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryNotifyIcon {
    /// can be used as a unique identifier of the registered tray icon
    pub key: String,
    pub executable_path: PathBuf,
    pub initial_tooltip: Option<String>,
    /// PNG image of the cached icon
    pub icon_snapshot: Vec<u8>,
    pub icon_guid: Option<String>,
    pub icon_uid: Option<u32>,
    pub is_promoted: bool,
    pub is_running: bool,
}
