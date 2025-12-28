pub mod application;
pub mod domain;
pub mod infrastructure;

use application::FullState;
use seelen_core::{
    resource::{PluginId, WidgetId},
    state::{
        value::{KnownPlugin, PluginValue},
        WegPinnedItemsVisibility, WegTemporalItemsVisibility, WindowManagerLayout, WorkspaceId,
    },
    system_state::MonitorId,
};
use uuid::Uuid;

use crate::resources::RESOURCES;

impl FullState {
    pub fn is_widget_enabled(&self, widget_id: &WidgetId) -> bool {
        self.settings.is_widget_enabled(widget_id)
    }

    pub fn is_widget_enable_on_monitor(
        &self,
        widget_id: &WidgetId,
        monitor_id: &MonitorId,
    ) -> bool {
        self.settings
            .is_widget_enabled_on_monitor(widget_id, monitor_id)
    }

    pub fn is_weg_enabled(&self) -> bool {
        self.is_widget_enabled(&WidgetId::known_weg())
    }

    #[allow(dead_code)]
    pub fn is_bar_enabled(&self) -> bool {
        self.is_widget_enabled(&WidgetId::known_toolbar())
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        self.is_widget_enabled(&WidgetId::known_wm())
    }

    pub fn is_weg_enabled_on_monitor(&self, monitor_id: &MonitorId) -> bool {
        self.is_widget_enable_on_monitor(&WidgetId::known_weg(), monitor_id)
    }

    pub fn is_bar_enabled_on_monitor(&self, monitor_id: &MonitorId) -> bool {
        self.is_widget_enable_on_monitor(&WidgetId::known_toolbar(), monitor_id)
    }

    pub fn is_window_manager_enabled_on_monitor(&self, monitor_id: &MonitorId) -> bool {
        self.is_widget_enable_on_monitor(&WidgetId::known_wm(), monitor_id)
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

    pub fn is_launcher_enabled(&self) -> bool {
        self.is_widget_enabled(&WidgetId::known_launcher())
    }

    pub fn is_wall_enabled(&self) -> bool {
        self.is_widget_enabled(&WidgetId::known_wall())
    }

    pub fn are_shortcuts_enabled(&self) -> bool {
        self.settings.shortcuts.enabled
    }

    pub fn get_wm_layout(&self, workspace_id: &WorkspaceId) -> WindowManagerLayout {
        let base = WindowManagerLayout::default();

        let layout_id = self.get_wm_layout_id(workspace_id);

        let mut plugin_with_layout = None;
        RESOURCES.plugins.any(|_, p| {
            if p.id == layout_id {
                plugin_with_layout = Some(p.clone());
                true
            } else {
                false
            }
        });

        let Some(plugin) = plugin_with_layout else {
            return base;
        };

        let PluginValue::Known(plugin) = &plugin.plugin else {
            return base;
        };

        let KnownPlugin::WManager(layout) = plugin else {
            return base;
        };

        layout.clone()
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
        self.settings.language.as_ref().unwrap()
    }
}
