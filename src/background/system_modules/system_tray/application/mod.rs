mod storage;
#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::{SysTrayIcon, SystrayIconAction};
use slu_ipc::messages::Win32TrayEvent;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::TracedMutex,
};

use super::domain::{
    SystemTrayEvent, SystemTraySystem, TrayIconConfig, apply_configs, reorder_configs, set_promoted,
};

#[cfg(windows)]
type Platform = windows::WindowsSystemTray;

/// The icons are tied to their owner windows, so the native layer keeps them, while the
/// user settings over them (pinned state and order) are kept here.
pub struct SystemTrayManager {
    platform: Platform,
    /// User settings of the icons, in display order.
    configs: TracedMutex<Vec<TrayIconConfig>>,
}

event_manager!(SystemTrayManager, SystemTrayEvent);

impl SystemTrayManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<SystemTrayManager> = LazyLock::new(|| {
            let mut m = SystemTrayManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            configs: TracedMutex::new(Vec::new()),
        }
    }

    fn init(&mut self) -> Result<()> {
        *self.configs.lock() = self.load_configs();
        self.platform.init(Self::send)
    }

    /// Stored settings, imported from the OS the first time.
    fn load_configs(&self) -> Vec<TrayIconConfig> {
        match storage::load() {
            Ok(Some(configs)) => return configs,
            Ok(None) => {}
            Err(err) => {
                log::error!("Failed to read the tray icons settings: {err}");
                return Vec::new();
            }
        }

        let configs = self.platform.legacy_icon_configs().unwrap_or_else(|err| {
            log::warn!("Tray icons settings of the OS not imported: {err}");
            Vec::new()
        });
        log::info!("Imported the settings of {} tray icons", configs.len());
        // stored right away so the import only happens once
        storage::save(&configs).log_error();
        configs
    }

    /// Handles a tray event received via IPC.
    pub fn handle_tray_event(event: Win32TrayEvent) {
        Self::instance().platform.handle_hook_event(event);
    }

    /// Icons in display order with their settings applied, without the ones of dead apps.
    pub fn icons(&self) -> Vec<SysTrayIcon> {
        if self.platform.prune_dead_icons() {
            // keep every other tray consumer in sync, not only this caller
            Self::send(SystemTrayEvent::Changed);
        }
        let mut icons = self.platform.icons();
        apply_configs(&mut icons, &self.configs.lock());
        icons
    }

    pub fn send_action(&self, session_id: &str, action: &SystrayIconAction) -> Result<()> {
        self.platform.send_action(session_id, action)
    }

    /// Shows the icon directly on the toolbar or only in the tray popup. The setting is
    /// stored by key, so it applies to every icon sharing it (e.g. all the instances of a
    /// multi-instance app).
    pub fn set_promoted(&self, session_id: &str, promoted: bool) -> Result<()> {
        let key = self
            .platform
            .icons()
            .iter()
            .find(|icon| icon.session_id == session_id)
            .ok_or("Icon not found")
            .map(TrayIconConfig::key_of)?
            .ok_or("Icon without key, its executable is unknown")?;
        {
            let mut configs = self.configs.lock();
            set_promoted(&mut configs, &key, promoted);
            storage::save(&configs)?;
        }
        Self::send(SystemTrayEvent::Changed);
        Ok(())
    }

    /// Reorders the given icons, the others keep their position. Icons without key are ignored.
    pub fn set_order(&self, session_ids: &[String]) -> Result<()> {
        let icons = self.platform.icons();
        let keys: Vec<String> = session_ids
            .iter()
            .filter_map(|id| icons.iter().find(|icon| &icon.session_id == id))
            .filter_map(TrayIconConfig::key_of)
            .collect();
        {
            let mut configs = self.configs.lock();
            reorder_configs(&mut configs, &keys);
            storage::save(&configs)?;
        }
        Self::send(SystemTrayEvent::Changed);
        Ok(())
    }
}
