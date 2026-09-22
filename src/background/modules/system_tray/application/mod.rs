pub mod tray_hook_loader;
pub mod tray_icon;
mod util;

use std::sync::{LazyLock, Once};

use seelen_core::system_state::{SysTrayIcon, SysTrayIconId};
use slu_ipc::messages::Win32TrayEvent;

use crate::{
    event_manager, hook::HookManager,
    modules::system_tray::application::tray_hook_loader::TrayHookLoader,
    utils::lock_free::SyncHashMap, windows_api::window::event::WinEvent,
};

pub struct SystemTrayManager {
    icons: SyncHashMap<SysTrayIconId, SysTrayIcon>,
    /// Process that owned each icon's window when it was registered, used to
    /// detect a recycled window handle.
    owners: SyncHashMap<SysTrayIconId, u32>,
    _loader: Option<TrayHookLoader>,
}

#[derive(Debug, Clone)]
pub enum SystemTrayEvent {
    Changed,
}

event_manager!(SystemTrayManager, SystemTrayEvent);

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
            owners: SyncHashMap::new(),
            _loader: loader,
        }
    }

    pub fn instance() -> &'static Self {
        static SYSTEM_TRAY_MANAGER: LazyLock<SystemTrayManager> =
            LazyLock::new(SystemTrayManager::create);
        static WIN_EVENTS: Once = Once::new();

        let manager = &*SYSTEM_TRAY_MANAGER;
        // Subscribed only after the manager is fully created: the callback calls
        // `instance()`, so doing it inside `create` could re-enter the LazyLock.
        WIN_EVENTS.call_once(|| {
            // Explorer drops the icons of dead windows without notifying the
            // hook, so react to the owner window being destroyed instead.
            HookManager::subscribe(|(event, window)| {
                if event == WinEvent::ObjectDestroy {
                    Self::on_window_destroyed(window.address());
                }
            });
        });
        manager
    }

    /// Drops the icons owned by a window that was just destroyed.
    fn on_window_destroyed(address: isize) {
        let manager = Self::instance();
        let owns_icon = manager
            .icons
            .any(|(_, icon)| icon.window_handle == Some(address));
        if owns_icon && manager.prune_dead_icons() {
            Self::send(SystemTrayEvent::Changed);
        }
    }

    /// Handles a tray event received via IPC
    /// This method should be called from the AppIpc handler
    pub fn handle_tray_event(event: Win32TrayEvent) {
        let manager = Self::instance();
        let changed = manager.process_event(event).is_some();
        let pruned = manager.prune_dead_icons();
        if changed || pruned {
            Self::send(SystemTrayEvent::Changed);
        }
    }
}
