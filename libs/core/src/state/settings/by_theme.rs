use std::collections::HashMap;

use crate::state::config::CssVariableName;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
pub struct ThemeSettings(HashMap<CssVariableName, String>);
