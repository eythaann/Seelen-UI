use schemars::JsonSchema;

use crate::{
    resource::WidgetId,
    state::{twm::TwmPlugin, ToolbarItem},
    utils::TsUnknown,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(untagged)]
pub enum PluginValue {
    Known(Box<KnownPlugin>),
    Any(ThirdPartyPlugin),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(tag = "target", content = "plugin")]
pub enum KnownPlugin {
    #[serde(rename = "@seelen/fancy-toolbar")]
    FacyToolbar(Box<ToolbarItem>),
    #[serde(rename = "@seelen/window-manager")]
    WManager(Box<TwmPlugin>),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct ThirdPartyPlugin {
    /// The friendly id of the widget that will use this plugin
    /// example: `@username/widget-name`
    target: WidgetId,
    /// The plugin data, this can be anything and depends on the widget using this plugin
    /// to handle it, parse it and use it.
    plugin: TsUnknown,
}
