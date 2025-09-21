use std::collections::HashMap;

use bincode::{Decode, Encode};
use seelen_core::rect::Rect;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Encode, Decode)]
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
}

/// Seelen UI Service Actions
#[allow(dead_code)]
#[derive(Debug, Clone, Encode, Decode)]
pub enum SvcAction {
    Stop,
    SetStartup(bool),
    /// this needs to be a string because of bincode's limitations
    /// this should be SluShortcutsSettings on json format
    SetShortcutsConfig(String),
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
        #[bincode(with_serde)]
        rect: Rect,
        flags: u32,
    },
    DeferWindowPositions {
        #[bincode(with_serde)]
        list: HashMap<isize, Rect>,
        animated: bool,
        animation_duration: u64,
        easing: String,
    },
    SetForeground(isize),
    StartShortcutRegistration,
    StopShortcutRegistration,
}

#[derive(Debug, Clone, Encode, Decode)]
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
}

// ========== Launcher ==========

#[derive(Debug, Clone, Encode, Decode)]
pub enum LauncherMessage {
    GuiStarted,
    Quit,
}
