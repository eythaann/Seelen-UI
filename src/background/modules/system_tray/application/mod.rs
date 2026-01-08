pub mod tray_hook_loader;
pub mod tray_icon;
mod util;

use std::sync::LazyLock;

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{SysTrayIcon, SysTrayIconId},
};
use slu_ipc::messages::Win32TrayEvent;

use crate::{
    app::emit_to_webviews, modules::system_tray::application::tray_hook_loader::TrayHookLoader,
    utils::lock_free::SyncHashMap,
};

pub struct SystemTrayManager {
    icons: SyncHashMap<SysTrayIconId, SysTrayIcon>,
    _loader: Option<TrayHookLoader>,
}

impl SystemTrayManager {
    fn create() -> Self {
        log::trace!("Creating system tray manager");

        let loader = match TrayHookLoader::new() {
            Ok(loader) => Some(loader),
            Err(err) => {
                log::error!("Failed to create tray hook loader: {:?}", err);
                None
            }
        };

        Self {
            icons: SyncHashMap::new(),
            _loader: loader,
        }
    }

    pub fn instance() -> &'static Self {
        static SYSTEM_TRAY_MANAGER: LazyLock<SystemTrayManager> =
            LazyLock::new(SystemTrayManager::create);
        &SYSTEM_TRAY_MANAGER
    }

    /// Handles a tray event received via IPC
    /// This method should be called from the AppIpc handler
    pub fn handle_tray_event(event: Win32TrayEvent) {
        if let Some(_event) = Self::instance().process_event(event) {
            emit_to_webviews(SeelenEvent::SystemTrayChanged, Self::instance().icons());
        }
    }
}
