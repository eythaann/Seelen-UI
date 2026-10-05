use std::collections::HashMap;

use seelen_core::{
    rect::Rect,
    state::{Settings, shortcuts::ResolvedShortcut},
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
    SetSettings(Box<Settings>),
    /// Sends the resolved shortcuts list to the service for hotkey registration.
    /// Pass an empty `Vec` to unregister all hotkeys (e.g. when shortcuts are disabled).
    SetShortcuts(Vec<ResolvedShortcut>),
    ShowWindow {
        hwnd: isize,
        command: i32,
    },
    ShowWindowAsync {
        hwnd: isize,
        command: i32,
    },
    SetWindowPosition {
        hwnd: isize,
        rect: Rect,
        flags: u32,
    },
    DeferWindowPositions {
        list: HashMap<isize, Rect>,
        animated: bool,
        animation_duration: u64,
        easing: String,
    },
    SetForeground(isize),
    StartShortcutRegistration,
    StopShortcutRegistration,
    HideNativeTaskbar,
    RestoreNativeTaskbar,
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
    pub is_visible: bool,
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
