use schemars::JsonSchema;

use crate::{
    resource::WidgetId,
    state::{twm::TwmPlugin, ToolbarItem},
    utils::TsUnknown,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged)]
pub enum PluginValue {
    Known(Box<KnownPlugin>),
    Any(ThirdPartyPlugin),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "target", content = "plugin")]
pub enum KnownPlugin {
    #[serde(rename = "@seelen/fancy-toolbar")]
    FacyToolbar(ToolbarItem),
    #[serde(rename = "@seelen/window-manager")]
    WManager(TwmPlugin),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct ThirdPartyPlugin {
    /// The friendly id of the widget that will use this plugin
    /// example: `@username/widget-name`
    target: WidgetId,
    /// The plugin data, this can be anything and depends on the widget using this plugin
    /// to handle it, parse it and use it.
    plugin: TsUnknown,
}
