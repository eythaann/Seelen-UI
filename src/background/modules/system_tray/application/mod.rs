pub mod tray_icon;
pub mod tray_spy;
mod util;

use std::sync::LazyLock;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{SysTrayIcon, SysTrayIconId},
};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error::{ErrorMap, ResultLogExt},
    modules::system_tray::application::tray_spy::TraySpy,
    utils::{lock_free::SyncHashMap, spawn_named_thread},
};

static SYSTEM_TRAY_MANAGER: LazyLock<SystemTrayManager> = LazyLock::new(SystemTrayManager::create);

pub struct SystemTrayManager {
    icons: SyncHashMap<SysTrayIconId, SysTrayIcon>,
    _spy: Option<TraySpy>,
}

impl SystemTrayManager {
    fn create() -> Self {
        let Ok((_spy, mut event_rx)) = TraySpy::new() else {
            return Self {
                icons: SyncHashMap::new(),
                _spy: None,
            };
        };

        spawn_named_thread("System Tray", move || {
            while let Some(event) = event_rx.blocking_recv() {
                if let Some(_event) = Self::instance().process_event(event) {
                    get_app_handle()
                        .emit(SeelenEvent::SystemTrayChanged, Self::instance().icons())
                        .wrap_error()
                        .log_error();
                }
            }
        });

        Self {
            icons: SyncHashMap::new(),
            _spy: Some(_spy),
        }
    }

    pub fn instance() -> &'static Self {
        &SYSTEM_TRAY_MANAGER
    }
}
