pub mod application;
pub mod domain;
pub mod infrastructure;

use std::collections::HashMap;

use application::FullState;
use domain::AhkVar;
use seelen_core::{
    resource::{PluginId, WidgetId},
    state::{WegPinnedItemsVisibility, WegTemporalItemsVisibility, Widget, WidgetInstanceType},
};
use uuid::Uuid;

use crate::windows_api::monitor::Monitor;

impl FullState {
    pub fn is_weg_enabled(&self) -> bool {
        self.settings.by_widget.weg.enabled
    }

    pub fn is_weg_enabled_on_monitor(&self, monitor_id: &str) -> bool {
        let is_global_enabled = self.is_weg_enabled();
        match self.settings.monitors_v3.get(monitor_id) {
            Some(config) => {
                is_global_enabled && config.by_widget.is_widget_enabled(&WidgetId::known_weg())
            }
            None => is_global_enabled,
        }
    }

    pub fn is_bar_enabled(&self) -> bool {
        self.settings.by_widget.fancy_toolbar.enabled
    }

    pub fn is_bar_enabled_on_monitor(&self, monitor: &Monitor) -> bool {
        let is_global_enabled = self.is_bar_enabled();
        let device_id = match monitor.stable_id() {
            Ok(id) => id,
            Err(_) => return is_global_enabled,
        };
        match self.settings.monitors_v3.get(&device_id) {
            Some(config) => {
                is_global_enabled
                    && config
                        .by_widget
                        .is_widget_enabled(&WidgetId::known_toolbar())
            }
            None => is_global_enabled,
        }
    }

    pub fn is_widget_enable(&self, widget: &Widget, monitor: &Monitor) -> bool {
        // new widgets are enabled by default
        let is_globally_enabled = self
            .settings
            .by_widget
            .others
            .get(&widget.id)
            .map_or(true, |config| config.enabled);

        if !is_globally_enabled {
            return false;
        }

        match widget.instances {
            WidgetInstanceType::ReplicaByMonitor => {
                let Ok(device_id) = monitor.stable_id() else {
                    return false;
                };

                self.settings
                    .monitors_v3
                    .get(&device_id)
                    .map_or(true, |monitor_config| {
                        monitor_config.by_widget.is_widget_enabled(&widget.id)
                    })
            }
            _ => monitor.is_primary(),
        }
    }

    pub fn get_widget_instances_ids(&self, widget_id: &WidgetId) -> Vec<Uuid> {
        let config = self.settings.by_widget.others.get(widget_id);
        match config {
            Some(config) => config
                .instances
                .as_ref()
                .map_or_else(Default::default, |i| i.keys().cloned().collect()),
            None => Vec::new(),
        }
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        self.settings.by_widget.wm.enabled
    }

    pub fn is_rofi_enabled(&self) -> bool {
        self.settings.by_widget.launcher.enabled
    }

    pub fn is_wall_enabled(&self) -> bool {
        self.settings.by_widget.wall.enabled
    }

    pub fn is_ahk_enabled(&self) -> bool {
        self.settings.ahk_enabled
    }

    pub fn get_ahk_variables(&self) -> HashMap<String, AhkVar> {
        self.settings.ahk_variables.as_hash_map()
    }

    pub fn get_wm_layout_id(&self, _monitor: &Monitor, _workspace_idx: usize) -> PluginId {
        let mut default = self.settings.by_widget.wm.default_layout.clone();
        if !default.is_valid() {
            default = "@default/wm-bspwm".into();
        }

        /* let Ok(id) = monitor.stable_id() else {
            return default;
        }; */

        /* let config = match self.settings.monitors_v3.get(&id) {
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
        } */

        default
    }

    pub fn get_weg_temporal_item_visibility(
        &self,
        _monitor_id: &str,
    ) -> WegTemporalItemsVisibility {
        /* match self.settings.monitors_v2.get(monitor_id) {
            Some(config) => config
                .by_widget
                .weg
                .temporal_items_visibility
                .unwrap_or(default),
            None => default,
        } */
        self.settings.by_widget.weg.temporal_items_visibility
    }

    pub fn get_weg_pinned_item_visibility(&self, _monitor_id: &str) -> WegPinnedItemsVisibility {
        /* match self.settings.monitors_v2.get(monitor_id) {
            Some(config) => config
                .by_widget
                .weg
                .pinned_items_visibility
                .unwrap_or(default),
            None => default,
        } */
        self.settings.by_widget.weg.pinned_items_visibility
    }

    pub fn locale(&self) -> &String {
        // always should be filled
        self.settings().language.as_ref().unwrap()
    }
}
