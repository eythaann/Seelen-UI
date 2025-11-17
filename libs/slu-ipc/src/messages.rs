use std::collections::HashMap;

use seelen_core::{rect::Rect, state::Settings};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    Success,
    Err(String),
}

impl IpcResponse {
    pub fn ok(self) -> Result<()> {
        match self {
            IpcResponse::Success => Ok(()),
            IpcResponse::Err(err) => Err(Error::IpcResponseError(err)),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

// ==============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AppMessage {
    /// Command-line messages
    Cli(Vec<String>),
    /// System tray change event
    TrayChanged(Win32TrayEvent),
    /// Debug message for logging and diagnostics
    Debug(String),
}

impl AppMessage {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvcMessage {
    pub token: String,
    pub action: SvcAction,
}

impl SvcMessage {
    pub fn signature() -> &'static str {
        std::env!("SLU_SERVICE_CONNECTION_TOKEN")
    }

    pub fn is_signature_valid(&self) -> bool {
        self.token == SvcMessage::signature()
    }

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
