use std::collections::HashMap;

use schemars::JsonSchema;
use uuid::Uuid;

use crate::{resource::WidgetId, utils::TsUnknown};

use super::{
    FancyToolbarSettings, SeelenLauncherSettings, SeelenWallSettings, SeelenWegSettings,
    WindowManagerSettings,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default)]
pub struct SettingsByWidget {
    #[serde(rename = "@seelen/weg")]
    pub weg: SeelenWegSettings,
    #[serde(rename = "@seelen/fancy-toolbar")]
    pub fancy_toolbar: FancyToolbarSettings,
    #[serde(rename = "@seelen/window-manager")]
    pub wm: WindowManagerSettings,
    #[serde(rename = "@seelen/wallpaper-manager")]
    pub wall: SeelenWallSettings,
    #[serde(rename = "@seelen/launcher")]
    pub launcher: SeelenLauncherSettings,
    #[serde(flatten)]
    pub others: HashMap<WidgetId, ThirdPartyWidgetSettings>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default)]
pub struct ThirdPartyWidgetSettings {
    /// Enable or disable the widget
    pub enabled: bool,
    /// By intance will be used to store settings in case of multiple instances allowed on widget.\
    /// The map values will be merged with the root object and default values on settings declaration.
    #[serde(rename = "$instances")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional = nullable)]
    pub instances: Option<HashMap<Uuid, HashMap<String, TsUnknown>>>,
    #[serde(flatten)]
    pub rest: HashMap<String, TsUnknown>,
}

impl Default for ThirdPartyWidgetSettings {
    fn default() -> Self {
        Self {
            enabled: true, // new widgets are enabled by default
            instances: None,
            rest: Default::default(),
        }
    }
}
