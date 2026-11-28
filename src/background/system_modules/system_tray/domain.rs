use std::{collections::HashMap, path::Path};

use seelen_core::system_state::{SysTrayIcon, SystrayIconAction};
use serde::{Deserialize, Serialize};
use slu_ipc::messages::Win32TrayEvent;

use crate::error::Result;

/// OS contract of the system tray (notification area icons) module.
pub trait SystemTraySystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts tracking the icons, reporting changes through `on_event`.
    /// Called once; the tracking lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(SystemTrayEvent) + Send + Sync + 'static;

    /// Applies an icon change reported by the tray hook (through the service IPC).
    fn handle_hook_event(&self, event: Win32TrayEvent);

    /// Removes the icons whose owner died without removing them.
    /// Returns whether any was removed (not reported as an event).
    fn prune_dead_icons(&self) -> bool;

    /// Live icons, unsorted and without the user settings applied (see `apply_configs`).
    fn icons(&self) -> Vec<SysTrayIcon>;

    fn send_action(&self, session_id: &str, action: &SystrayIconAction) -> Result<()>;

    /// Pinned state and order the OS kept for the icons, in display order. Only read once,
    /// to import them when Seelen has no settings stored yet.
    fn legacy_icon_configs(&self) -> Result<Vec<TrayIconConfig>>;
}

#[derive(Debug, Clone)]
pub enum SystemTrayEvent {
    Changed,
}

/// User settings of an icon, persisted by Seelen. The position of the config in the
/// stored list is the position of the icon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayIconConfig {
    /// See `TrayIconConfig::key_for`.
    pub key: String,
    /// Shown directly on the toolbar, otherwise only in the tray popup.
    #[serde(default)]
    pub is_promoted: bool,
}

impl TrayIconConfig {
    /// Persistent key of an icon, the same as the shell uses to remember it: its guid, or the
    /// executable that registered it plus its uid. Every instance of a multi-instance app
    /// shares the key (and so its settings).
    pub fn key_for(
        executable_path: &Path,
        uid: Option<u32>,
        guid: Option<uuid::Uuid>,
    ) -> Option<String> {
        if let Some(guid) = guid {
            return Some(guid.to_string());
        }
        let uid = uid?;
        if executable_path.as_os_str().is_empty() {
            return None;
        }
        Some(format!(
            "{}::{uid}",
            executable_path.to_string_lossy().to_lowercase()
        ))
    }

    pub fn key_of(icon: &SysTrayIcon) -> Option<String> {
        Self::key_for(&icon.executable_path, icon.uid, icon.guid)
    }
}

/// Applies the user settings over the icons: sets their key and pinned state, and sorts them
/// as stored. Icons without settings are not pinned and go at the end, sorted by key.
pub fn apply_configs(icons: &mut [SysTrayIcon], configs: &[TrayIconConfig]) {
    let positions: HashMap<&str, usize> = configs
        .iter()
        .enumerate()
        .map(|(position, config)| (config.key.as_str(), position))
        .collect();

    for icon in icons.iter_mut() {
        icon.key = TrayIconConfig::key_of(icon);
        icon.is_promoted = icon
            .key
            .as_deref()
            .and_then(|key| positions.get(key))
            .is_some_and(|position| configs[*position].is_promoted);
    }

    let position = |icon: &SysTrayIcon| {
        icon.key
            .as_deref()
            .and_then(|key| positions.get(key).copied())
            .unwrap_or(usize::MAX)
    };
    icons.sort_by(|a, b| {
        position(a)
            .cmp(&position(b))
            .then_with(|| a.key.cmp(&b.key))
            .then_with(|| a.session_id.cmp(&b.session_id))
    });
}

/// Sets the pinned state of the key, adding its config at the end if missing.
pub fn set_promoted(configs: &mut Vec<TrayIconConfig>, key: &str, promoted: bool) {
    match configs.iter_mut().find(|config| config.key == key) {
        Some(config) => config.is_promoted = promoted,
        None => configs.push(TrayIconConfig {
            key: key.to_string(),
            is_promoted: promoted,
        }),
    }
}

/// Sorts the configs following the sequence of `keys` without overriding the others: the
/// slots occupied by the given keys are refilled in the order of `keys`, every other config
/// keeps its position. Keys without config are added at the end first, repeated keys are ignored.
pub fn reorder_configs(configs: &mut Vec<TrayIconConfig>, keys: &[String]) {
    let mut sorted: Vec<&String> = Vec::with_capacity(keys.len());
    for key in keys {
        if !sorted.contains(&key) {
            sorted.push(key);
        }
    }

    for key in &sorted {
        if !configs.iter().any(|config| &config.key == *key) {
            configs.push(TrayIconConfig {
                key: key.to_string(),
                is_promoted: false,
            });
        }
    }

    let slots: Vec<usize> = configs
        .iter()
        .enumerate()
        .filter(|(_, config)| sorted.contains(&&config.key))
        .map(|(index, _)| index)
        .collect();
    let mut by_key: HashMap<String, TrayIconConfig> = slots
        .iter()
        .map(|index| (configs[*index].key.clone(), configs[*index].clone()))
        .collect();
    for (index, key) in slots.into_iter().zip(sorted) {
        if let Some(config) = by_key.remove(key) {
            configs[index] = config;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn configs(list: &[(&str, bool)]) -> Vec<TrayIconConfig> {
        list.iter()
            .map(|(key, is_promoted)| TrayIconConfig {
                key: key.to_string(),
                is_promoted: *is_promoted,
            })
            .collect()
    }

    fn keys(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn icon(session_id: &str, exe: &str, uid: u32) -> SysTrayIcon {
        SysTrayIcon {
            session_id: session_id.to_string(),
            key: None,
            executable_path: PathBuf::from(exe),
            uid: Some(uid),
            window_handle: None,
            guid: None,
            tooltip: String::new(),
            icon_handle: None,
            icon_path: None,
            icon_image_hash: None,
            is_glyph: false,
            callback_message: None,
            version: None,
            is_visible: true,
            is_promoted: false,
        }
    }

    #[test]
    fn key_prefers_guid_and_ignores_path_case() {
        let guid = uuid::Uuid::from_u128(1);
        assert_eq!(
            TrayIconConfig::key_for(Path::new("C:\\App.exe"), Some(3), Some(guid)),
            Some(guid.to_string())
        );
        assert_eq!(
            TrayIconConfig::key_for(Path::new("C:\\Apps\\QQ.exe"), Some(3), None),
            Some("c:\\apps\\qq.exe::3".to_string())
        );
        assert_eq!(TrayIconConfig::key_for(Path::new(""), Some(3), None), None);
        assert_eq!(
            TrayIconConfig::key_for(Path::new("C:\\App.exe"), None, None),
            None
        );
    }

    #[test]
    fn applies_settings_and_sorts_unknown_icons_last() {
        let mut icons = vec![
            icon("unknown", "c:\\z.exe", 1),
            icon("b", "c:\\b.exe", 1),
            icon("a", "c:\\a.exe", 1),
            icon("b2", "c:\\b.exe", 1),
        ];
        apply_configs(
            &mut icons,
            &configs(&[("c:\\a.exe::1", false), ("c:\\b.exe::1", true)]),
        );

        let order: Vec<&str> = icons.iter().map(|i| i.session_id.as_str()).collect();
        assert_eq!(order, ["a", "b", "b2", "unknown"]);
        let promoted: Vec<bool> = icons.iter().map(|i| i.is_promoted).collect();
        assert_eq!(promoted, [false, true, true, false]);
        assert_eq!(icons[3].key.as_deref(), Some("c:\\z.exe::1"));
    }

    #[test]
    fn sets_promoted_adding_missing_config() {
        let mut list = configs(&[("a", false)]);
        set_promoted(&mut list, "a", true);
        set_promoted(&mut list, "b", true);
        assert_eq!(list, configs(&[("a", true), ("b", true)]));
    }

    #[test]
    fn reorders_only_given_keys_in_place() {
        let mut list = configs(&[("a", false), ("b", true), ("c", false), ("d", false)]);
        reorder_configs(&mut list, &keys(&["d", "b", "d"]));
        assert_eq!(
            list,
            configs(&[("a", false), ("d", false), ("c", false), ("b", true)])
        );
    }

    #[test]
    fn reorder_adds_missing_keys_at_the_end() {
        let mut list = configs(&[("a", false), ("b", false)]);
        reorder_configs(&mut list, &keys(&["x", "a"]));
        assert_eq!(list, configs(&[("x", false), ("b", false), ("a", false)]));
    }
}
