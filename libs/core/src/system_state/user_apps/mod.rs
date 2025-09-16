use std::path::PathBuf;

use crate::system_state::MonitorId;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FocusedApp {
    pub hwnd: isize,
    pub monitor: MonitorId,
    pub title: String,
    pub name: String,
    pub exe: Option<PathBuf>, // todo remove this and refactor weg items to a shared windows state
    pub umid: Option<String>, // todo remove this and refactor weg items to a shared windows state
    pub is_maximized: bool,
    pub is_fullscreened: bool,
    pub is_seelen_overlay: bool,
    pub is_being_dragged: bool,
}
