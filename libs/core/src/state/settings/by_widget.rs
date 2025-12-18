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

impl SettingsByWidget {
    pub fn is_enabled(&self, widget_id: &WidgetId) -> bool {
        match widget_id.as_str() {
            "@seelen/weg" => self.weg.enabled,
            "@seelen/fancy-toolbar" => self.fancy_toolbar.enabled,
            "@seelen/window-manager" => self.wm.enabled,
            "@seelen/wallpaper-manager" => self.wall.enabled,
            "@seelen/launcher" => self.launcher.enabled,
            _ => match self.others.get(widget_id) {
                Some(settings) => settings.enabled,
                // only official widgets are enabled by default
                None => widget_id.starts_with("@seelen"),
            },
        }
    }

    pub fn set_enabled(&mut self, widget_id: &WidgetId, enabled: bool) {
        match widget_id.as_str() {
            "@seelen/weg" => self.weg.enabled = enabled,
            "@seelen/fancy-toolbar" => self.fancy_toolbar.enabled = enabled,
            "@seelen/window-manager" => self.wm.enabled = enabled,
            "@seelen/wallpaper-manager" => self.wall.enabled = enabled,
            "@seelen/launcher" => self.launcher.enabled = enabled,
            _ => match self.others.entry(widget_id.clone()) {
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    o.get_mut().enabled = enabled;
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(ThirdPartyWidgetSettings {
                        enabled,
                        ..Default::default()
                    });
                }
            },
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
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
