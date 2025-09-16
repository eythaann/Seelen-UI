use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use crate::{resource::WidgetId, state::by_widget::ThirdPartyWidgetSettings};

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, TS)]
pub struct MonitorSettingsByWidget(HashMap<WidgetId, ThirdPartyWidgetSettings>);

impl MonitorSettingsByWidget {
    pub fn is_widget_enabled(&self, widget_id: &WidgetId) -> bool {
        self.0
            .get(widget_id)
            .is_none_or(|settings| settings.enabled)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct MonitorConfiguration {
    /// dictionary of settings by widget
    pub by_widget: MonitorSettingsByWidget,
}
