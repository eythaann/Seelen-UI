#[cfg(test)]
mod tests;

pub mod config;

use std::collections::HashMap;

use config::ThemeSettingsDefinition;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    resource::{ResourceKind, ResourceMetadata, SluResource, ThemeId, WidgetId},
    system_state::Color,
};

pub static ALLOWED_STYLE_EXTENSIONS: &[&str] = &["css", "scss", "sass"];

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
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
    pub shared_styles: Option<String>,
    /// Default design tokens that can be consumed by applications that do not
    /// support CSS.
    pub tokens: Option<ThemeTokens>,
    /// Design tokens used when the application is in dark mode.
    /// If omitted, `tokens` will be used instead.
    pub tokens_dark: Option<ThemeTokens>,
    /// Override the system accent color. Alpha value is ignored.
    pub accent_override: Option<Color>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeTokens {
    pub foreground_color: Option<Color>,
    pub foreground_secondary_color: Option<Color>,
    pub foreground_muted_color: Option<Color>,
    pub foreground_disabled_color: Option<Color>,

    pub background_color: Option<Color>,
    pub background_light_color: Option<Color>,
    pub background_dark_color: Option<Color>,

    pub shadow_small: Option<ShadowToken>,
    pub shadow_medium: Option<ShadowToken>,
    pub shadow_large: Option<ShadowToken>,
}
impl ThemeTokens {
    pub fn merge(self, other: Self) -> Self {
        Self {
            foreground_color: other.foreground_color.or(self.foreground_color),
            foreground_secondary_color: other
                .foreground_secondary_color
                .or(self.foreground_secondary_color),
            foreground_muted_color: other.foreground_muted_color.or(self.foreground_muted_color),
            foreground_disabled_color: other
                .foreground_disabled_color
                .or(self.foreground_disabled_color),

            background_color: other.background_color.or(self.background_color),
            background_light_color: other.background_light_color.or(self.background_light_color),
            background_dark_color: other.background_dark_color.or(self.background_dark_color),

            shadow_small: other.shadow_small.or(self.shadow_small),
            shadow_medium: other.shadow_medium.or(self.shadow_medium),
            shadow_large: other.shadow_large.or(self.shadow_large),
        }
    }

    pub fn merged(&self, other: &Self) -> Self {
        self.clone().merge(other.clone())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "salvo", derive(salvo::oapi::ToSchema))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct ShadowToken {
    pub blur: f32,
    pub spread: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub color: Color,
}
