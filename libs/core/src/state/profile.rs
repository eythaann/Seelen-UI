use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Placeholder;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSettings {
    themes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Profile {
    name: String,
    toolbar_layout: Placeholder,
    settings: ProfileSettings,
}
