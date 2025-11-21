use std::path::PathBuf;

use crate::{rect::Rect, system_state::MonitorId};

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FocusedApp {
    pub hwnd: isize,
    pub monitor: MonitorId,
    pub title: String,
    pub class: String,
    pub name: String,
    pub exe: Option<PathBuf>,
    pub umid: Option<String>,
    pub is_maximized: bool,
    pub is_fullscreened: bool,
    pub is_seelen_overlay: bool,
    /// this is the rect of the window, without the shadow.
    pub rect: Option<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UserApplication {
    pub name: String,
    pub path: PathBuf,
    pub umid: Option<String>,
    pub is_in_start_menu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UserAppWindow {
    pub hwnd: isize,
    pub monitor: MonitorId,
    pub title: String,
    pub app_name: String,
    pub is_zoomed: bool,
    pub is_iconic: bool,
    pub is_fullscreen: bool,
    /// this can be from the window property store, or inherited from the process
    pub umid: Option<String>,
    /// if the window is a frame, this information will be mapped to the process creator
    pub process: ProcessInformation,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ProcessInformation {
    pub id: u32,
    pub path: Option<PathBuf>,
}
