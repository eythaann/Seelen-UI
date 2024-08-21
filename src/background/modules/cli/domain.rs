use seelen_core::state::{Placeholder, Theme, WindowManagerLayout};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    /// Url of wallpaper
    pub wallpaper: Option<String>,
    pub resources: ResourceItems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceItems {
    pub theme: Option<Theme>,
    pub placeholder: Option<Placeholder>,
    pub layout: Option<WindowManagerLayout>,
}
