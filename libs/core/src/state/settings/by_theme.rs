use std::collections::HashMap;

use crate::state::config::CssVariableName;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct ThemeSettings(HashMap<CssVariableName, String>);
