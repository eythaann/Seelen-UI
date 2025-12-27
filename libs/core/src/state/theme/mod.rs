#[cfg(test)]
mod tests;

pub mod config;

use std::{collections::HashMap, path::Path};

use config::ThemeSettingsDefinition;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    error::Result,
    resource::{ResourceKind, ResourceMetadata, SluResource, ThemeId, WidgetId},
    utils::search_resource_entrypoint,
};

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

    fn load_from_folder(path: &Path) -> Result<Theme> {
        let mut theme = Self::load_old_folder_schema(path)?;

        'outer: for entry in path.read_dir()?.flatten() {
            let outer_path = entry.path();
            if !outer_path.is_dir() {
                let (Some(file_stem), Some(ext)) = (outer_path.file_stem(), outer_path.extension())
                else {
                    continue 'outer;
                };

                if file_stem == "shared" && ALLOWED_STYLE_EXTENSIONS.iter().any(|e| *e == ext) {
                    let css = if ext == "scss" || ext == "sass" {
                        grass::from_path(&outer_path, &grass::Options::default())?
                    } else {
                        std::fs::read_to_string(&outer_path)?
                    };
                    theme.shared_styles = css;
                }
                continue 'outer;
            }

            let creator_username = entry.file_name();
            'inner: for entry in outer_path.read_dir()?.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue 'inner;
                }

                let (Some(resource_name), Some(ext)) = (path.file_stem(), path.extension()) else {
                    continue 'inner;
                };

                if ALLOWED_STYLE_EXTENSIONS.iter().any(|e| *e == ext) {
                    let css = if ext == "scss" || ext == "sass" {
                        grass::from_path(&path, &grass::Options::default())?
                    } else {
                        std::fs::read_to_string(&path)?
                    };
                    theme.styles.insert(
                        WidgetId::from(
                            format!(
                                "@{}/{}",
                                creator_username.to_string_lossy(),
                                resource_name.to_string_lossy()
                            )
                            .as_str(),
                        ),
                        css,
                    );
                }
            }
        }
        Ok(theme)
    }
}

impl Theme {
    /// Load theme from a folder using old deprecated paths since v2.1.0 will be removed in v3
    fn load_old_folder_schema(path: &Path) -> Result<Theme> {
        let file = search_resource_entrypoint(path).unwrap_or_else(|| {
            path.join("theme.yml") // backward compatibility to be removed in v3
        });
        let mut theme = Self::load_from_file(&file)?;

        if path.join("theme.weg.css").exists() {
            theme.styles.insert(
                WidgetId::known_weg(),
                std::fs::read_to_string(path.join("theme.weg.css"))?,
            );
        }
        if path.join("theme.toolbar.css").exists() {
            theme.styles.insert(
                WidgetId::known_toolbar(),
                std::fs::read_to_string(path.join("theme.toolbar.css"))?,
            );
        }
        if path.join("theme.wm.css").exists() {
            theme.styles.insert(
                WidgetId::known_wm(),
                std::fs::read_to_string(path.join("theme.wm.css"))?,
            );
        }
        if path.join("theme.wall.css").exists() {
            theme.styles.insert(
                WidgetId::known_wall(),
                std::fs::read_to_string(path.join("theme.wall.css"))?,
            );
        };
        Ok(theme)
    }
}
