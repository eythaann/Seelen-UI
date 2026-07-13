use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct WegPluginItem {
    pub scopes: HashSet<String>,
    /// JS function definition for content to display in tooltip of the item.
    pub tooltip: Option<String>,
    /// JS function definition that will be executed when the item is clicked.
    pub on_click: Option<String>,
}
