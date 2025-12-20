use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, state::VirtualDesktops, system_state::MonitorId};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    virtual_desktops::SluWorkspacesManager2,
};

fn get_vd_manager() -> &'static SluWorkspacesManager2 {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SluWorkspacesManager2::subscribe(|_event| {
            let payload: VirtualDesktops = SluWorkspacesManager2::instance().into();
            get_app_handle()
                .emit(SeelenEvent::VirtualDesktopsChanged, payload)
                .log_error();
        });
    });

    SluWorkspacesManager2::instance()
}

#[tauri::command(async)]
pub fn get_virtual_desktops() -> VirtualDesktops {
    get_vd_manager().into()
}

#[tauri::command(async)]
pub fn switch_workspace(monitor_id: MonitorId, idx: usize) -> Result<()> {
    get_vd_manager().switch_to(&monitor_id, idx)
}
