use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct WegPluginItem {
    pub scopes: HashSet<String>,
    // JS function definition for content to display in the item.
    pub render: String,
    /// If true, `render` is expected to return a custom icon key (string) instead of
    /// drawing on the canvas. The item will be displayed using that custom icon.
    pub no_canvas: bool,
    /// JS function definition for content to display in tooltip of the item.
    pub tooltip: Option<String>,
    /// JS function definition for content to display in badge of the item.
    pub badge: Option<String>,
    /// JS function definition that will be executed when the item is clicked.
    pub on_click: Option<String>,
}
