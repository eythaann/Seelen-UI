use std::collections::HashMap;

use seelen_core::{
    rect::Rect,
    state::shortcuts::ResolvedShortcut,
    system_state::{WindowHandle, WindowState},
};
use serde::{Deserialize, Serialize};

use crate::{
    commands::AppCommand,
    error::{Error, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    Success,
    Err(String),
}

impl IpcResponse {
    pub fn ok(self) -> Result<()> {
        match self {
            IpcResponse::Success => Ok(()),
            IpcResponse::Err(err) => Err(Error::IpcResponse(err)),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(Error::IpcResponse(
                "No response received (server may have crashed or disconnected)".to_string(),
            ));
        }
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

// ==============================================

#[derive(Debug, Serialize, Deserialize)]
pub enum AppMessage {
    /// Raw command line messages
    Cli(Vec<String>),
    Command(AppCommand),
    /// Open a URI or file path in the main instance
    OpenUri(String),
    /// System tray change event
    TrayChanged(Win32TrayEvent),
    /// Taskbar button change event (`ITaskbarList3` calls captured by the hook)
    TaskbarButtonChanged(Win32TaskbarButtonEvent),
    /// Debug message for logging and diagnostics
    Debug(String),
}

impl AppMessage {
    /// Messages sent by the hook DLL from inside explorer.exe, they don't need a meaningful
    /// response so they are acknowledged before being processed.
    pub fn is_notification(&self) -> bool {
        matches!(
            self,
            AppMessage::TrayChanged(_) | AppMessage::TaskbarButtonChanged(_) | AppMessage::Debug(_)
        )
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

// ==============================================

/// Seelen UI Service Actions
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SvcAction {
    Stop,
    SetStartup(bool),
    /// Sends the resolved shortcuts list to the service for hotkey registration.
    /// Pass an empty `Vec` to unregister all hotkeys (e.g. when shortcuts are disabled).
    SetShortcuts(Vec<ResolvedShortcut>),
    // The serialized names of the window variants (`hwnd`, `command`, `flags`) are kept from when
    // they carried raw Win32 values, so the app and the service stay compatible across versions.
    ShowWindow {
        #[serde(rename = "hwnd")]
        window: WindowHandle,
        #[serde(rename = "command")]
        state: WindowState,
    },
    /// Same as [`SvcAction::ShowWindow`] but without waiting for the window to process it.
    ShowWindowAsync {
        #[serde(rename = "hwnd")]
        window: WindowHandle,
        #[serde(rename = "command")]
        state: WindowState,
    },
    /// Moves and resizes the window, without activating it nor changing its z-order.
    SetWindowPosition {
        #[serde(rename = "hwnd")]
        window: WindowHandle,
        rect: Rect,
        /// Deprecated, ignored: only sent (always `0`) because previous versions require it.
        #[serde(default)]
        flags: u32,
    },
    DeferWindowPositions {
        list: HashMap<WindowHandle, Rect>,
        animated: bool,
        animation_duration: u64,
        easing: String,
    },
    SetForeground(WindowHandle),
    StartShortcutRegistration,
    StopShortcutRegistration,
    HideNativeTaskbar,
    RestoreNativeTaskbar,
    /// Mirrors the Seelen UI wallpaper on the native desktop wallpaper, so system surfaces that
    /// show it (UAC prompt, lock/logon transitions, etc.) match what the user sees.
    /// `path: None` sets a black wallpaper. The user's original wallpaper is saved on the first
    /// call and restored by `RestoreNativeWallpaper` or when the service stops.
    SetNativeWallpaper {
        path: Option<std::path::PathBuf>,
    },
    RestoreNativeWallpaper,
}

impl SvcAction {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

// ========== Launcher ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LauncherMessage {
    GuiStarted,
    Quit,
}

impl LauncherMessage {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

// ========== Tray ==========

/// System tray icon data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IconEventData {
    pub uid: Option<u32>,
    pub window_handle: Option<isize>,
    pub guid: Option<uuid::Uuid>,
    pub tooltip: Option<String>,
    pub icon_handle: Option<isize>,
    pub callback_message: Option<u32>,
    pub version: Option<u32>,
    /// `NIS_HIDDEN` inverted, `None` if the message doesn't change it.
    pub is_visible: Option<bool>,
    /// `NIS_SHAREDICON`: the icon reuses the image of another icon with the same `HICON`.
    /// `None` if the message doesn't change it.
    pub shared_icon: Option<bool>,
}

/// System tray events captured by the hook
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Win32TrayEvent {
    IconAdd { data: IconEventData },
    IconUpdate { data: IconEventData },
    IconRemove { data: IconEventData },
}

// ========== Taskbar Buttons ==========

/// `ITaskbarList3` calls sent by applications to the taskband, captured by the hook
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Win32TaskbarButtonEvent {
    /// `SetOverlayIcon` icon part, `icon_handle` is None when the overlay was removed.
    OverlayIconChanged {
        hwnd: isize,
        icon_handle: Option<isize>,
    },
    /// `SetOverlayIcon` description part (accessibility text, e.g. "3 unread messages").
    OverlayDescriptionChanged {
        hwnd: isize,
        description: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// JSON as sent by previous versions, when these variants carried raw Win32 values.
    #[test]
    fn reads_window_actions_of_previous_versions() -> Result<()> {
        let action = SvcAction::from_bytes(br#"{"ShowWindowAsync":{"hwnd":1234,"command":6}}"#)?;
        assert!(matches!(
            action,
            SvcAction::ShowWindowAsync {
                window: 1234,
                state: WindowState::Minimized
            }
        ));

        let action = SvcAction::from_bytes(
            br#"{"SetWindowPosition":{"hwnd":-5,"rect":{"left":1,"top":2,"right":3,"bottom":4},"flags":0}}"#,
        )?;
        assert!(matches!(
            action,
            SvcAction::SetWindowPosition { window: -5, rect, .. } if rect.right == 3
        ));

        let action = SvcAction::from_bytes(
            br#"{"DeferWindowPositions":{"list":{"42":{"left":0,"top":0,"right":10,"bottom":10}},"animated":true,"animation_duration":200,"easing":"Linear"}}"#,
        )?;
        assert!(
            matches!(action, SvcAction::DeferWindowPositions { list, .. } if list.contains_key(&42))
        );

        let action = SvcAction::from_bytes(br#"{"SetForeground":99}"#)?;
        assert!(matches!(action, SvcAction::SetForeground(99)));
        Ok(())
    }

    /// Previous versions must be able to read what this one sends.
    #[test]
    fn writes_window_actions_as_previous_versions() -> Result<()> {
        let bytes = SvcAction::ShowWindow {
            window: 1234,
            state: WindowState::Restored,
        }
        .to_bytes()?;
        assert_eq!(bytes, br#"{"ShowWindow":{"hwnd":1234,"command":9}}"#);

        let bytes = SvcAction::SetWindowPosition {
            window: 7,
            rect: Rect::default(),
            flags: 0,
        }
        .to_bytes()?;
        let json: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(json["SetWindowPosition"]["hwnd"], 7);
        assert_eq!(json["SetWindowPosition"]["flags"], 0);
        Ok(())
    }
}
