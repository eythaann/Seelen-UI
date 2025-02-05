use serde::{Deserialize, Serialize};

/// Seelen UI Service Actions
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SvcAction {
    Stop,
    SetStartup(bool),
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
