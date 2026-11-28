mod tray_hook_loader;
mod tray_icon;
mod util;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use seelen_core::system_state::{SysTrayIcon, SystrayIconAction, WindowHandle};
use slu_ipc::messages::Win32TrayEvent;
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, KEY_READ},
};

use crate::{
    error::Result,
    platform::hook::{HookManager, WindowEvent},
    system_modules::system_tray::domain::{SystemTrayEvent, SystemTraySystem, TrayIconConfig},
    utils::lock_free::SyncHashMap,
    windows_api::WindowsApi,
};

use self::{tray_hook_loader::TrayHookLoader, tray_icon::SavedIconImage};

/// Settings Explorer keeps for the icons (Windows 11 only), read once to import them.
const NOTIFY_ICON_SETTINGS: &str = "Control Panel\\NotifyIconSettings";

type EventCallback = Box<dyn Fn(SystemTrayEvent) + Send + Sync>;

pub struct WindowsSystemTray {
    state: Arc<TrayState>,
    /// Injects the hook reporting the icons to the service. Dropping it removes the hook.
    _loader: Option<TrayHookLoader>,
}

impl SystemTraySystem for WindowsSystemTray {
    fn create() -> Self {
        Self {
            state: Arc::new(TrayState {
                icons: SyncHashMap::new(),
                owners: SyncHashMap::new(),
                images: SyncHashMap::new(),
                on_event: OnceLock::new(),
            }),
            _loader: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(SystemTrayEvent) + Send + Sync + 'static,
    {
        let _ = self.state.on_event.set(Box::new(on_event));

        // Explorer drops the icons of dead windows without notifying the
        // hook, so react to the owner window being destroyed instead.
        let state = self.state.clone();
        HookManager::subscribe(move |(event, window)| {
            if event == WindowEvent::Destroyed {
                state.on_window_destroyed(window.handle());
            }
        });

        log::trace!("Creating system tray hook loader");
        self._loader = Some(TrayHookLoader::new()?);
        Ok(())
    }

    fn handle_hook_event(&self, event: Win32TrayEvent) {
        self.state.handle_event(event);
    }

    fn prune_dead_icons(&self) -> bool {
        self.state.prune_dead_icons()
    }

    fn icons(&self) -> Vec<SysTrayIcon> {
        self.state.icons.values()
    }

    fn send_action(&self, session_id: &str, action: &SystrayIconAction) -> Result<()> {
        self.state.send_action(session_id, action)
    }

    fn legacy_icon_configs(&self) -> Result<Vec<TrayIconConfig>> {
        let settings = RegKey::predef(HKEY_CURRENT_USER).open_subkey(NOTIFY_ICON_SETTINGS)?;
        let order = get_registry_ui_order().unwrap_or_default();

        let mut entries: Vec<RegistryNotifyIcon> = settings
            .enum_keys()
            .flatten()
            .filter_map(|key| read_registry_notify_icon(&settings, &key).ok())
            .collect();
        entries.sort_by_key(|entry| {
            order
                .iter()
                .position(|key| *key == entry.key)
                .unwrap_or(usize::MAX)
        });

        let mut configs: Vec<TrayIconConfig> = Vec::with_capacity(entries.len());
        for entry in entries {
            let Some(key) =
                TrayIconConfig::key_for(&entry.executable_path, entry.icon_uid, entry.icon_guid)
            else {
                continue;
            };
            if configs.iter().any(|config| config.key == key) {
                continue;
            }
            configs.push(TrayIconConfig {
                key,
                is_promoted: entry.is_promoted,
            });
        }
        Ok(configs)
    }
}

/// Known icons, shared with the hook subscription.
pub(super) struct TrayState {
    /// Icons by their session id.
    icons: SyncHashMap<String, SysTrayIcon>,
    /// Process that owned each icon's window when it was registered, used to
    /// detect a recycled window handle.
    owners: SyncHashMap<String, u32>,
    /// Icon images already written to disk, by owner (see `image_owner_key`) and `HICON`.
    /// Apps like Task Manager switch between a few hidden icons on every update, so
    /// each `HICON` is converted and written only once.
    images: SyncHashMap<String, HashMap<isize, SavedIconImage>>,
    on_event: OnceLock<EventCallback>,
}

impl TrayState {
    fn notify_changed(&self) {
        if let Some(on_event) = self.on_event.get() {
            on_event(SystemTrayEvent::Changed);
        }
    }

    /// Drops the icons owned by a window that was just destroyed.
    fn on_window_destroyed(&self, address: WindowHandle) {
        let owns_icon = self
            .icons
            .any(|(_, icon)| icon.window_handle.is_some_and(|w| w.handle == address));
        if owns_icon && self.prune_dead_icons() {
            self.notify_changed();
        }
    }

    /// Handles a tray event reported by the hook.
    fn handle_event(&self, event: Win32TrayEvent) {
        let changed = self.process_event(event).is_some();
        let pruned = self.prune_dead_icons();
        if changed || pruned {
            self.notify_changed();
        }
    }
}

/// Entry of an icon under `NotifyIconSettings`.
struct RegistryNotifyIcon {
    key: String,
    executable_path: PathBuf,
    icon_guid: Option<uuid::Uuid>,
    icon_uid: Option<u32>,
    is_promoted: bool,
}

fn read_registry_notify_icon(settings: &RegKey, key: &str) -> Result<RegistryNotifyIcon> {
    let regkey = settings.open_subkey_with_flags(key, KEY_READ)?;
    let path_with_guid: String = regkey.get_value("ExecutablePath")?;
    Ok(RegistryNotifyIcon {
        key: key.to_string(),
        executable_path: WindowsApi::resolve_guid_path(path_with_guid)?,
        icon_guid: regkey
            .get_value::<String, _>("IconGuid")
            .ok()
            .and_then(|guid| uuid::Uuid::parse_str(&guid).ok()),
        icon_uid: regkey.get_value("UID").ok(),
        is_promoted: regkey
            .get_value::<u32, _>("IsPromoted")
            .is_ok_and(|v| v != 0),
    })
}

/// Registry keys of the icons in the order they are displayed on the Win Taskbar and Win Tray Overflow.
fn get_registry_ui_order() -> Result<Vec<String>> {
    let settings = RegKey::predef(HKEY_CURRENT_USER).open_subkey(NOTIFY_ICON_SETTINGS)?;
    let list = settings.get_raw_value("UIOrderList")?.bytes;
    Ok(list
        .chunks_exact(8)
        .map(|chunk| {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(chunk);
            u64::from_le_bytes(bytes).to_string()
        })
        .collect())
}
