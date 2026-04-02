#[cfg(test)]
mod tests;

pub mod config;

use std::collections::HashMap;

use config::ThemeSettingsDefinition;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::resource::{ResourceKind, ResourceMetadata, SluResource, ThemeId, WidgetId};

pub static ALLOWED_STYLE_EXTENSIONS: &[&str] = &["css", "scss", "sass"];

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Theme {
    pub id: ThemeId,
    /// Metadata about the theme
    #[serde(alias = "info")] // for backwards compatibility before v2.0
    pub metadata: ResourceMetadata,
    pub settings: ThemeSettingsDefinition,
    /// Css Styles of the theme
    pub styles: HashMap<WidgetId, String>,
    /// Shared css styles for all widgets, commonly used to set styles
    /// for the components library
    pub shared_styles: String,
}

impl SluResource for Theme {
    const KIND: ResourceKind = ResourceKind::Theme;

    fn metadata(&self) -> &ResourceMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut ResourceMetadata {
        &mut self.metadata
    }
}
