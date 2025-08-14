use seelen_core::state::VirtualDesktops;

use crate::virtual_desktops::get_vd_manager;

#[tauri::command(async)]
pub fn get_virtual_desktops() -> VirtualDesktops {
    get_vd_manager().desktops().clone()
}
