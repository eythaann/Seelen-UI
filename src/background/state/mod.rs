pub mod application;
pub mod domain;
pub mod infrastructure;

use application::FullState;
use seelen_core::{
    resource::{PluginId, WidgetId},
    state::{
        WegPinnedItemsVisibility, WegTemporalItemsVisibility, Widget, WidgetInstanceType,
        WorkspaceId,
    },
    system_state::MonitorId,
};
use uuid::Uuid;

impl FullState {
    pub fn is_weg_enabled(&self) -> bool {
        self.settings.by_widget.weg.enabled
    }

    pub fn is_weg_enabled_on_monitor(&self, monitor_id: &MonitorId) -> bool {
        let is_global_enabled = self.is_weg_enabled();
        match self.settings.monitors_v3.get(monitor_id.as_str()) {
            Some(config) => {
                is_global_enabled && config.by_widget.is_widget_enabled(&WidgetId::known_weg())
            }
            None => is_global_enabled,
        }
    }

    pub fn is_bar_enabled(&self) -> bool {
        self.settings.by_widget.fancy_toolbar.enabled
    }

    pub fn is_bar_enabled_on_monitor(&self, monitor_id: &MonitorId) -> bool {
        let is_global_enabled = self.is_bar_enabled();
        match self.settings.monitors_v3.get(monitor_id.as_str()) {
            Some(config) => {
                is_global_enabled
                    && config
                        .by_widget
                        .is_widget_enabled(&WidgetId::known_toolbar())
            }
            None => is_global_enabled,
        }
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        self.settings.by_widget.wm.enabled
    }

    pub fn is_window_manager_enabled_on_monitor(&self, monitor_id: &MonitorId) -> bool {
        let is_global_enabled = self.is_window_manager_enabled();
        match self.settings.monitors_v3.get(monitor_id.as_str()) {
            Some(config) => {
                is_global_enabled && config.by_widget.is_widget_enabled(&WidgetId::known_wm())
            }
            None => is_global_enabled,
        }
    }

    pub fn is_widget_enable_on_monitor(&self, widget: &Widget, monitor_id: &MonitorId) -> bool {
        // new widgets are enabled by default
        let is_globally_enabled = self
            .settings
            .by_widget
            .others
            .get(&widget.id)
            .is_none_or(|config| config.enabled);

        if !is_globally_enabled {
            return false;
        }

        match widget.instances {
            WidgetInstanceType::ReplicaByMonitor => self
                .settings
                .monitors_v3
                .get(monitor_id.as_str())
                .is_none_or(|monitor_config| {
                    monitor_config.by_widget.is_widget_enabled(&widget.id)
                }),
            _ => false,
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

    pub fn is_rofi_enabled(&self) -> bool {
        self.settings.by_widget.launcher.enabled
    }

    pub fn is_wall_enabled(&self) -> bool {
        self.settings.by_widget.wall.enabled
    }

    pub fn are_shortcuts_enabled(&self) -> bool {
        self.settings.shortcuts.enabled
    }

    pub fn get_wm_layout_id(&self, _workspace_id: &WorkspaceId) -> PluginId {
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
        _monitor_id: &MonitorId,
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

    pub fn get_weg_pinned_item_visibility(
        &self,
        _monitor_id: &MonitorId,
    ) -> WegPinnedItemsVisibility {
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
