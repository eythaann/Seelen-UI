use std::path::PathBuf;

use serde::Serialize;
use windows::Win32::UI::Shell::{
    NOTIFYICONDATAW_0, NOTIFY_ICON_DATA_FLAGS, NOTIFY_ICON_INFOTIP_FLAGS, NOTIFY_ICON_MESSAGE,
    NOTIFY_ICON_STATE,
};

use crate::windows_api::string_utils::WindowsString;

/// Tray message sent to `Shell_TrayWnd` and intercepted by our spy window.
#[repr(C)]
pub struct ShellTrayMessage {
    pub magic_number: i32,
    pub message: NOTIFY_ICON_MESSAGE,
    pub data: NotifyIconData,
    pub version: u32,
}

/// Contains the data for a system tray icon.
///
/// When `Shell_NotifyIcon` sends its message to `Shell_Traywnd`, it
/// actually uses a 32-bit handle for the window and icons. This makes it slightly
/// different than the `windows` crate's `NOTIFYICONDATAW` type when building for x64.
///
/// https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataa
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_snake_case)]
pub struct NotifyIconData {
    callback_size: u32,
    window_handle: u32,
    uid: u32,
    flags: NOTIFY_ICON_DATA_FLAGS,
    callback_message: u32,
    icon_handle: u32,
    tooltip: [u16; 128],
    state: NOTIFY_ICON_STATE,
    state_mask: NOTIFY_ICON_STATE,
    info: [u16; 256],
    anonymous: NOTIFYICONDATAW_0,
    info_title: [u16; 64],
    info_flags: NOTIFY_ICON_INFOTIP_FLAGS,
    guid_item: windows_core::GUID,
    balloon_icon_handle: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct NotifyIconIdentifier {
    magic_number: i32,
    message: i32,
    callback_size: i32,
    padding: i32,
    window_handle: u32,
    uid: u32,
    guid_item: windows_core::GUID,
}

/// Identifier for a systray icon.
///
/// A systray icon is either identified by a (window handle + uid) or
/// its guid. Since a systray icon can be updated to also include a
/// guid or window handle/uid later on, a stable ID is useful for
/// consistently identifying an icon.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub enum TrayIconId {
    HandleUid(isize, u32),
    Guid(String),
}

impl std::fmt::Display for TrayIconId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrayIconId::HandleUid(handle, uid) => write!(f, "{:x}:{}", handle, uid),
            TrayIconId::Guid(guid) => write!(f, "{}", guid),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayIconV2 {
    pub id: TrayIconId,
    pub title: String,
    pub info: String,
    pub tooltip: String,
}

impl From<NotifyIconData> for TrayIconV2 {
    fn from(data: NotifyIconData) -> Self {
        let id = if data.guid_item != Default::default() {
            TrayIconId::Guid(format!("{:?}", data.guid_item))
        } else {
            TrayIconId::HandleUid(data.window_handle as _, data.uid)
        };

        Self {
            id,
            tooltip: WindowsString::from_slice(&data.tooltip).to_string(),
            info: WindowsString::from_slice(&data.info).to_string(),
            title: WindowsString::from_slice(&data.info_title).to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayIcon {
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
    pub icon_snapshot: Option<Vec<u8>>,
    pub icon_guid: Option<String>,
    pub icon_uid: Option<u32>,
    pub is_promoted: bool,
    pub is_running: bool,
}
