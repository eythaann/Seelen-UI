pub mod tray_hook_loader;
pub mod tray_icon;
mod util;

use std::{
    path::{Path, PathBuf},
    sync::{LazyLock, Once},
};

use seelen_core::system_state::SysTrayIcon;
use slu_ipc::messages::{IconEventData, Win32TrayEvent};
use winreg::{
    RegKey, RegValue,
    enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_BINARY},
};

use crate::{
    error::Result,
    event_manager,
    hook::HookManager,
    modules::system_tray::application::tray_hook_loader::TrayHookLoader,
    utils::{lock_free::SyncHashMap, resolve_guid_path},
    windows_api::window::event::WinEvent,
};

pub struct SystemTrayManager {
    /// Icons by their registry key.
    icons: SyncHashMap<String, SysTrayIcon>,
    /// Process that owned each icon's window when it was registered, used to
    /// detect a recycled window handle.
    owners: SyncHashMap<String, u32>,
    /// Icons reported by the hook whose registry entry wasn't written yet, by
    /// their session identity (see `session_identity`).
    pending: SyncHashMap<String, IconEventData>,
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
            pending: SyncHashMap::new(),
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

    /// Returns all icons managed by the `Systray`, sorted as Windows displays them
    /// (`UIOrderList`). Icons missing from that list are placed at the end.
    pub fn icons(&self) -> Vec<SysTrayIcon> {
        let mut icons = self.icons.values();
        let order = get_registry_ui_order().unwrap_or_else(|err| {
            log::warn!("Failed to read tray icons UI order: {err:?}");
            Vec::new()
        });
        icons.sort_by_key(|icon| {
            order
                .iter()
                .position(|key| *key == icon.registry_key)
                .unwrap_or(usize::MAX)
        });
        icons
    }

    /// Reorders the icons on the registry UI order following `keys`, entries not included
    /// in `keys` (like icons of closed apps) keep their position.
    pub fn set_order(&self, keys: &[String]) -> Result<()> {
        let mut order = get_registry_ui_order()?;
        sort_ui_order(&mut order, keys);
        set_registry_ui_order(&order)?;
        Self::send(SystemTrayEvent::Changed);
        Ok(())
    }

    /// Shows the icon directly on the taskbar or only in the tray overflow.
    pub fn set_promoted(&self, registry_key: &str, promoted: bool) -> Result<()> {
        if !self.icons.contains_key(registry_key) {
            return Err("Icon not found".into());
        }
        set_registry_notify_icon_promoted(registry_key, promoted)?;
        self.icons.with_lock(|icons| {
            if let Some(icon) = icons.get_mut(registry_key) {
                icon.is_promoted = promoted;
            }
        });
        Self::send(SystemTrayEvent::Changed);
        Ok(())
    }
}

const NOTIFY_ICON_SETTINGS: &str = "Control Panel\\NotifyIconSettings";

#[derive(Debug)]
#[allow(dead_code)]
pub struct RegistryNotifyIcon {
    pub key: String,
    pub executable_path: PathBuf,
    // pub icon_snapshot: Option<Vec<u8>>,
    pub initial_tooltip: Option<String>,
    pub icon_guid: Option<uuid::Uuid>,
    pub icon_uid: Option<u32>,
    pub is_promoted: bool,
}

fn read_registry_notify_icon(settings: &RegKey, key: &str) -> Result<RegistryNotifyIcon> {
    let regkey = settings.open_subkey_with_flags(key, KEY_READ)?;
    let path_with_guid: String = regkey.get_value("ExecutablePath")?;
    Ok(RegistryNotifyIcon {
        key: key.to_string(),
        executable_path: resolve_guid_path(path_with_guid)?,
        // icon_snapshot: regkey.get_raw_value("IconSnapShot").map(|v| v.bytes).ok(),
        initial_tooltip: regkey.get_value("InitialTooltip").ok(),
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

/// Sets whether the icon is shown on the taskbar (1) or only in the tray overflow (0).
fn set_registry_notify_icon_promoted(key: &str, promoted: bool) -> Result<()> {
    let regkey = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(NOTIFY_ICON_SETTINGS)?
        .open_subkey_with_flags(key, KEY_SET_VALUE)?;
    regkey.set_value("IsPromoted", &(promoted as u32))?;
    Ok(())
}

/// Finds the entry that Windows persisted for an icon. Icons with guid are identified by it,
/// otherwise by the executable that registered them plus their uid.
///
/// Note: Explorer writes the entry while handling `NIM_ADD`, so for an icon seen for the first
/// time it may not exist yet when the hook reports it.
pub fn find_registry_notify_icon(
    executable_path: Option<&Path>,
    guid: Option<uuid::Uuid>,
    uid: Option<u32>,
) -> Option<RegistryNotifyIcon> {
    let settings = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(NOTIFY_ICON_SETTINGS)
        .ok()?;
    let executable_path = executable_path.map(|p| p.to_string_lossy().to_lowercase());

    settings
        .enum_keys()
        .flatten()
        .filter_map(|key| read_registry_notify_icon(&settings, &key).ok())
        .find(|entry| match guid {
            Some(guid) => entry.icon_guid == Some(guid),
            None => {
                entry.icon_guid.is_none()
                    && uid.is_some()
                    && entry.icon_uid == uid
                    && executable_path.as_ref().is_some_and(|path| {
                        *path == entry.executable_path.to_string_lossy().to_lowercase()
                    })
            }
        })
}

/// Finds the entry of an icon without guid by its executable alone, when the executable
/// has exactly one such entry.
///
/// Explorer can keep a single entry per executable with the uid of an older run and never
/// write one for the current uid, so `find_registry_notify_icon` never matches. It happens
/// with apps that use their window handle as uid (MSI Afterburner) and with WinForms apps,
/// which give each `NotifyIcon` a new id (DSX).
pub fn find_registry_notify_icon_by_executable(
    executable_path: &Path,
) -> Option<RegistryNotifyIcon> {
    let settings = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(NOTIFY_ICON_SETTINGS)
        .ok()?;
    let executable_path = executable_path.to_string_lossy().to_lowercase();

    let mut entries = settings
        .enum_keys()
        .flatten()
        .filter_map(|key| read_registry_notify_icon(&settings, &key).ok())
        .filter(|entry| {
            entry.icon_guid.is_none()
                && entry.executable_path.to_string_lossy().to_lowercase() == executable_path
        });
    let entry = entries.next()?;
    entries.next().is_none().then_some(entry)
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

/// Writes the order in which icons are displayed on the Win Taskbar and Win Tray Overflow.
fn set_registry_ui_order(order: &[String]) -> Result<()> {
    let settings = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(NOTIFY_ICON_SETTINGS, KEY_SET_VALUE)?;
    let mut bytes = Vec::with_capacity(order.len() * 8);
    for key in order {
        bytes.extend_from_slice(&key.parse::<u64>()?.to_le_bytes());
    }
    settings.set_raw_value(
        "UIOrderList",
        &RegValue {
            bytes,
            vtype: REG_BINARY,
        },
    )?;
    Ok(())
}

/// Sorts `order` following the sequence of `keys` without overriding it: the slots occupied by
/// the given keys are refilled in the order of `keys`, every other entry keeps its position.
/// Keys not present in `order` and repeated keys are ignored.
fn sort_ui_order(order: &mut [String], keys: &[String]) {
    let mut sorted: Vec<String> = Vec::with_capacity(keys.len());
    for key in keys {
        if order.contains(key) && !sorted.contains(key) {
            sorted.push(key.clone());
        }
    }

    let mut next = sorted.iter();
    for slot in order.iter_mut() {
        if sorted.contains(slot)
            && let Some(key) = next.next()
        {
            *slot = key.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::sort_ui_order;

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn sorts_only_given_keys_in_place() {
        let mut order = strings(&["a", "b", "c", "d", "e"]);
        sort_ui_order(&mut order, &strings(&["d", "x", "b", "d"]));
        assert_eq!(order, strings(&["a", "d", "c", "b", "e"]));
    }
}
