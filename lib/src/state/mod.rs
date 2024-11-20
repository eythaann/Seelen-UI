mod icon_pack;
mod placeholder;
mod settings;
mod settings_by_app;
mod settings_by_monitor;
mod theme;
mod weg_items;
mod wm_layout;
mod plugin;
mod widget;

pub use icon_pack::*;
pub use placeholder::*;
pub use settings::*;
pub use settings_by_app::*;
pub use settings_by_monitor::*;
pub use theme::*;
pub use weg_items::*;
pub use wm_layout::*;
pub use plugin::*;
pub use widget::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct ResourceMetadata {
    pub display_name: String,
    pub author: String,
    pub description: String,
    pub filename: String,
    pub tags: Vec<String>,
}

impl Default for ResourceMetadata {
    fn default() -> Self {
        Self {
            display_name: "Unknown".to_string(),
            author: "Unknown".to_string(),
            description: String::new(),
            filename: String::new(),
            tags: Vec::new(),
        }
    }
}
