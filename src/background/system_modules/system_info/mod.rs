mod application;
mod domain;

use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{Core, Disk, Gpu, Memory, NetworkStatistics},
};

use crate::app::emit_to_webviews;

use self::{application::SystemInfo, domain::SystemInfoEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static SystemInfo {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        SystemInfo::subscribe(|event| {
            let info = SystemInfo::instance();
            match event {
                SystemInfoEvent::DisksChanged => {
                    emit_to_webviews(SeelenEvent::SystemDisksChanged, info.disks());
                }
                SystemInfoEvent::NetworkChanged => {
                    emit_to_webviews(SeelenEvent::SystemNetworkChanged, info.network());
                }
                SystemInfoEvent::MemoryChanged => {
                    emit_to_webviews(SeelenEvent::SystemMemoryChanged, info.memory());
                }
                SystemInfoEvent::CoresChanged => {
                    emit_to_webviews(SeelenEvent::SystemCoresChanged, info.cores());
                }
                SystemInfoEvent::GpusChanged => {
                    emit_to_webviews(SeelenEvent::SystemGpusChanged, info.gpus());
                }
            }
        });
    });
    SystemInfo::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_system_disks() -> Vec<Disk> {
        get_manager().disks()
    }

    pub fn get_system_network() -> Vec<NetworkStatistics> {
        get_manager().network()
    }

    pub fn get_system_memory() -> Memory {
        get_manager().memory()
    }

    pub fn get_system_cores() -> Vec<Core> {
        get_manager().cores()
    }

    pub fn get_system_gpus() -> Vec<Gpu> {
        get_manager().gpus()
    }
}
