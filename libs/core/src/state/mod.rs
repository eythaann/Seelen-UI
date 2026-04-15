mod icon_pack;
mod placeholder;
mod plugin;
mod popups;
pub mod settings;
mod theme;
mod wallpaper;
mod weg_items;
mod widget;
mod wm_layout;
mod workspaces;

pub use icon_pack::*;
pub use placeholder::*;
pub use plugin::*;
pub use popups::*;
pub use settings::*;
pub use theme::*;
pub use wallpaper::*;
pub use weg_items::*;
pub use widget::*;
pub use wm_layout::*;
pub use workspaces::*;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsBackup {
    pub data: serde_json::Value,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
