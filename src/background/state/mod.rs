pub mod application;
pub mod domain;
pub mod infrastructure;

use std::collections::HashMap;

use application::FullState;
use domain::AhkVar;
use seelen_core::state::{PluginId, WegPinnedItemsVisibility, WegTemporalItemsVisibility};

use crate::windows_api::monitor::Monitor;

impl FullState {
    pub fn is_weg_enabled(&self) -> bool {
        self.settings.seelenweg().enabled
    }

    pub fn is_weg_enabled_on_monitor(&self, monitor_id: &str) -> bool {
        let is_global_enabled = self.is_weg_enabled();
        match self.settings.monitors_v2.get(monitor_id) {
            Some(config) => is_global_enabled && config.weg().enabled,
            None => is_global_enabled,
        }
    }

    pub fn is_bar_enabled(&self) -> bool {
        self.settings.fancy_toolbar().enabled
    }

    pub fn is_bar_enabled_on_monitor(&self, monitor: &Monitor) -> bool {
        let is_global_enabled = self.is_bar_enabled();
        let device_id = match monitor.display_device() {
            Ok(device) => device.id,
            Err(_) => return is_global_enabled,
        };
        match self.settings.monitors_v2.get(&device_id) {
            Some(config) => is_global_enabled && config.tb().enabled,
            None => is_global_enabled,
        }
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        self.settings.window_manager().enabled
    }

    pub fn is_rofi_enabled(&self) -> bool {
        self.settings.launcher().enabled
    }

    pub fn is_wall_enabled(&self) -> bool {
        self.settings.wall().enabled
    }

    pub fn is_ahk_enabled(&self) -> bool {
        self.settings.ahk_enabled
    }

    pub fn get_ahk_variables(&self) -> HashMap<String, AhkVar> {
        self.settings.ahk_variables.as_hash_map()
    }

    pub fn get_wm_layout_id(&self, monitor: &Monitor, workspace_idx: usize) -> PluginId {
        let mut default = self.settings.window_manager().default_layout;
        if !default.is_valid() {
            default = "@default/wm-bspwm".into();
        }

        let device_id = match monitor.display_device() {
            Ok(device) => device.id,
            Err(_) => return default,
        };

        let config = match self.settings.monitors_v2.get(&device_id) {
            Some(config) => config,
            None => return default,
        };

        let workspace = match config.workspaces_v2.get(workspace_idx) {
            Some(workspace) => workspace,
            None => return default,
        };

        match &workspace.layout {
            Some(layout_id) => {
                let layout_id: PluginId = layout_id.as_str().into();
                if layout_id.is_valid() {
                    layout_id
                } else {
                    default
                }
            }
            None => default,
        }
    }

    pub fn get_weg_temporal_item_visibility(&self, monitor_id: &str) -> WegTemporalItemsVisibility {
        let default = self.settings.seelenweg().temporal_items_visibility;
        match self.settings.monitors_v2.get(monitor_id) {
            Some(config) => config.weg().temporal_items_visibility.unwrap_or(default),
            None => default,
        }
    }

    pub fn get_weg_pinned_item_visibility(&self, monitor_id: &str) -> WegPinnedItemsVisibility {
        let default = self.settings.seelenweg().pinned_items_visibility;
        match self.settings.monitors_v2.get(monitor_id) {
            Some(config) => config.weg().pinned_items_visibility.unwrap_or(default),
            None => default,
        }
    }
}
