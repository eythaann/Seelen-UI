pub mod application;
pub mod domain;
pub mod infrastructure;

use std::collections::HashMap;

use application::FullState;
use domain::AhkVar;

impl FullState {
    pub fn is_weg_enabled(&self) -> bool {
        self.settings.seelenweg.enabled
    }

    pub fn is_weg_enabled_on_monitor(&self, _monitor_idx: usize) -> bool {
        /* match self.settings.monitors.get(monitor_idx) {
            Some(monitor) => is_global_enabled && monitor.weg.enabled,
            None => is_global_enabled,
        } */
        self.is_weg_enabled()
    }

    pub fn is_bar_enabled(&self) -> bool {
        self.settings.fancy_toolbar.enabled
    }

    pub fn is_bar_enabled_on_monitor(&self, _monitor_idx: usize) -> bool {
        /* match self.settings.monitors.get(monitor_idx) {
            Some(monitor) => is_global_enabled && monitor.tb.enabled,
            None => is_global_enabled,
        } */
        self.is_bar_enabled()
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        self.settings.window_manager.enabled
    }

    pub fn is_rofi_enabled(&self) -> bool {
        self.settings.launcher.enabled
    }

    pub fn is_wall_enabled(&self) -> bool {
        self.settings.wall.enabled
    }

    pub fn is_ahk_enabled(&self) -> bool {
        self.settings.ahk_enabled
    }

    pub fn get_ahk_variables(&self) -> HashMap<String, AhkVar> {
        self.settings.ahk_variables.as_hash_map()
    }

    pub fn get_wm_layout_id(&self, _monitor_idx: usize, _workspace_idx: usize) -> String {
        /* match self.settings.monitors.get(monitor_idx) {
            Some(monitor) => match monitor.workspaces_v2.get(workspace_idx) {
                Some(workspace) => workspace.layout.clone().unwrap_or(default),
                None => default,
            },
            None => default,
        } */
        self.settings.window_manager.default_layout.clone()
    }
}
