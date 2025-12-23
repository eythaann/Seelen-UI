use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use crate::{
    resource::WidgetId,
    state::{by_widget::ThirdPartyWidgetSettings, WorkspaceId},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct MonitorSettingsByWidget(HashMap<WidgetId, ThirdPartyWidgetSettings>);

impl MonitorSettingsByWidget {
    pub fn is_widget_enabled(&self, widget_id: &WidgetId) -> bool {
        self.0
            .get(widget_id)
            .is_none_or(|settings| settings.enabled)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct MonitorConfiguration {
    /// dictionary of settings by widget
    pub by_widget: MonitorSettingsByWidget,
    /// Id of the wallpaper collection to use in this monitor.\
    /// If not set, the default wallpaper collection will be used.
    pub wallpaper_collection: Option<uuid::Uuid>,
    /// dictionary of settings by workspace on this monitor
    pub by_workspace: HashMap<WorkspaceId, WorkspaceConfiguration>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WorkspaceConfiguration {
    /// Id of the wallpaper collection to use in this workspace.\
    /// If not set, the monitor's wallpaper collection will be used.
    pub wallpaper_collection: Option<uuid::Uuid>,
}
