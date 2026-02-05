use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Core, Disk, Memory, NetworkStatistics},
};

use crate::{
    app::emit_to_webviews,
    modules::system::{SystemInfo, SystemInfoEvent},
};

/// Wrapper for SystemInfo that automatically registers Tauri events on first access
/// This keeps Tauri logic separate from system logic while ensuring lazy initialization
fn get_system_info() -> &'static SystemInfo {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SystemInfo::subscribe(|event| match event {
            SystemInfoEvent::DisksChanged => {
                emit_to_webviews(
                    SeelenEvent::SystemDisksChanged,
                    &*SystemInfo::instance().last_disks.lock(),
                );
            }
            SystemInfoEvent::NetworkChanged => {
                emit_to_webviews(
                    SeelenEvent::SystemNetworkChanged,
                    &*SystemInfo::instance().last_networks.lock(),
                );
            }
            SystemInfoEvent::MemoryChanged => {
                emit_to_webviews(
                    SeelenEvent::SystemMemoryChanged,
                    &*SystemInfo::instance().last_memory.lock(),
                );
            }
            SystemInfoEvent::CoresChanged => {
                emit_to_webviews(
                    SeelenEvent::SystemCoresChanged,
                    &*SystemInfo::instance().last_cores.lock(),
                );
            }
        });
    });

    SystemInfo::instance()
}

#[tauri::command(async)]
pub fn get_system_disks() -> Vec<Disk> {
    get_system_info().last_disks.lock().clone()
}

#[tauri::command(async)]
pub fn get_system_network() -> Vec<NetworkStatistics> {
    get_system_info().last_networks.lock().clone()
}

#[tauri::command(async)]
pub fn get_system_memory() -> Memory {
    get_system_info().last_memory.lock().clone()
}

#[tauri::command(async)]
pub fn get_system_cores() -> Vec<Core> {
    get_system_info().last_cores.lock().clone()
}
