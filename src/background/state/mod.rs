pub mod application;
pub mod infrastructure;

use application::AppSettings;
use seelen_core::{
    resource::{PluginId, WidgetId},
    state::{
        WorkspaceId,
        by_widget::GenericWidgetSettings,
        twm::TwmPlugin,
        value::{KnownPlugin, PluginValue},
    },
    system_state::MonitorId,
};
use uuid::Uuid;

use crate::{resources::RESOURCES, virtual_desktops::VdManager};

/// Key of the layout setting declared on the window manager widget metadata
const WM_LAYOUT_SETTING_KEY: &str = "layout";

impl AppSettings {
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

    pub fn get_wm_layout(&self, workspace_id: &WorkspaceId) -> TwmPlugin {
        let base = TwmPlugin::default();

        let layout_id = self.get_wm_layout_id(workspace_id);

        let mut plugin_with_layout = None;
        RESOURCES.plugins.any_sync(|_, p| {
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

        let KnownPlugin::WManager(layout) = &**plugin else {
            return base;
        };

        *layout.clone()
    }

    /// Priority: workspace layout → monitor layout → global layout → bspwm
    pub fn get_wm_layout_id(&self, workspace_id: &WorkspaceId) -> PluginId {
        let wm_id = WidgetId::known_wm();
        let from_overrides = |settings: Option<&GenericWidgetSettings>| -> Option<PluginId> {
            let value = settings?.rest.get(WM_LAYOUT_SETTING_KEY)?;
            let layout_id: PluginId = value.0.as_str()?.into();
            layout_id.is_valid().then_some(layout_id)
        };

        let monitor_config = VdManager::instance()
            .get_monitor_of_workspace(workspace_id)
            .and_then(|monitor_id| self.settings.monitors_v3.get(&monitor_id));

        if let Some(monitor_config) = monitor_config {
            let by_workspace = monitor_config
                .by_workspace
                .get(workspace_id)
                .and_then(|config| from_overrides(config.by_widget.get(&wm_id)));
            if let Some(layout_id) = by_workspace {
                return layout_id;
            }

            if let Some(layout_id) = from_overrides(monitor_config.by_widget.get(&wm_id)) {
                return layout_id;
            }
        }

        let default = self.settings.by_widget.wm.layout.clone();
        if default.is_valid() {
            default
        } else {
            "@default/wm-bspwm".into()
        }
    }

    pub fn locale(&self) -> &String {
        &self.settings.language
    }
}
