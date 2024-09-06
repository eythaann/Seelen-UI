use crate::{seelen::SEELEN, trace_lock};

use super::SeelenRofiApp;

#[tauri::command(async)]
pub fn launcher_get_apps() -> Vec<SeelenRofiApp> {
    if let Some(rofi) = trace_lock!(SEELEN).rofi() {
        return rofi.apps.clone();
    }
    Vec::new()
}
