use seelen_core::system_state::StartMenuItem;

use crate::{seelen::SEELEN, trace_lock};

#[tauri::command(async)]
pub fn launcher_get_apps() -> Vec<StartMenuItem> {
    if let Some(rofi) = trace_lock!(SEELEN).rofi() {
        return rofi.apps.clone();
    }
    Vec::new()
}
