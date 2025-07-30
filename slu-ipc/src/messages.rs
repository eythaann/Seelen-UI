use seelen_core::state::shortcuts::SluShortcutsSettings;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Seelen UI Service Actions
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SvcAction {
    Stop,
    SetStartup(bool),
    SetShortcutsConfig(SluShortcutsSettings),
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
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        flags: u32,
    },
    SetForeground(isize),
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
}

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
}
