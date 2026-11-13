use std::path::PathBuf;

use super::Color;
use crate::{
    rect::Rect,
    system_state::{BadgeValue, MonitorId},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum ZOrder {
    TopMost,
    NoTopMost,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct FocusedApp {
    pub hwnd: isize,
    pub owner_hwnd: isize,
    pub monitor: MonitorId,
    pub title: String,
    pub class: String,
    pub name: String,
    pub exe: Option<PathBuf>,
    pub umid: Option<String>,
    pub is_maximized: bool,
    pub is_fullscreened: bool,
    /// this is the rect of the window, without the shadow.
    pub rect: Option<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct UserAppWindow {
    pub hwnd: isize,
    pub monitor: MonitorId,
    pub title: String,
    pub app_name: String,
    pub is_zoomed: bool,
    pub is_iconic: bool,
    pub is_fullscreen: bool,
    /// this can be from the window property store, or inherited from the process
    pub umid: Option<String>,
    /// if the window is a frame, this information will be mapped to the process creator
    pub process: ProcessInformation,
    /// this app window can not be pinned
    pub prevent_pinning: bool,
    /// custom method to create start this application
    pub relaunch: Option<Relaunch>,
    /// rect of the window without shadow, in screen coordinates
    pub rect: Option<Rect>,
    /// unix timestamp (ms) of the last time this window was brought to the foreground;
    /// 0 means it has never been focused since tracking started.
    /// clients that want z-order-like sorting should sort descending by this field.
    pub last_foreground_at: i64,
    /// attention requested by the window via `FlashWindowEx`, cleared when it gets activated.
    pub attention: WindowAttention,
    /// overlay icon set on the taskbar button via `ITaskbarList3::SetOverlayIcon`,
    /// like the unread messages counter of chat apps.
    pub badge_icon_path: Option<PathBuf>,
    /// unix timestamp (ms) of the last time the badge was set, 0 if there is no badge.
    /// on grouped windows the taskbar shows the most recently set badge.
    pub badge_updated_at: i64,
    /// badge notification of the app (`BadgeUpdateManager`), by umid, e.g. unread messages count.
    pub badge_value: Option<BadgeValue>,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(repr(enum = name)))]
pub enum WindowAttention {
    /// the window is not requesting attention
    #[default]
    None,
    /// the window is actively flashing
    Flashing,
    /// the window stopped flashing but still waits to be activated, like the
    /// orange taskbar button left after a flash limited by `uCount`
    Highlighted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct ProcessInformation {
    pub id: u32,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct UserAppWindowPreview {
    pub hash: String,
    pub data: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct WindowPeekStatus {
    pub active: bool,
    pub committed: bool,
    pub hwnd: Option<isize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
pub struct WindowPreviewTarget {
    pub hwnd: isize,
    /// Full layout rectangle, in physical pixels relative to the destination client area.
    pub rect: Rect,
    /// Visible destination bounds. Keep `rect` intact so clipping preserves source alignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(optional))]
    pub clip: Option<Rect>,
}

#[derive(Debug, Default, Clone, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), ts(export))]
#[serde(rename_all = "camelCase")]
pub struct LiveWindowPreviewState {
    pub live: Vec<isize>,
    pub visible: bool,
    pub pointer_inside: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
pub struct UserAppWindowColors {
    pub top: Vec<Color>,
    pub bottom: Vec<Color>,
    pub left: Vec<Color>,
    pub right: Vec<Color>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(default, rename_all = "camelCase")]
pub struct Relaunch {
    /// program to be executed
    pub command: String,
    /// arguments to be passed to the relaunch program
    pub args: Option<RelaunchArguments>,
    /// path where ejecute the relaunch command
    pub working_dir: Option<PathBuf>,
    /// custom relaunch/window icon
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[serde(untagged)]
pub enum RelaunchArguments {
    Array(Vec<String>),
    String(String),
}

impl std::fmt::Display for RelaunchArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = match self {
            RelaunchArguments::String(args) => args.clone(),
            RelaunchArguments::Array(args) => args.join(" ").trim().to_owned(),
        };
        write!(f, "{}", args)
    }
}

#[cfg(test)]
mod preview_target_tests {
    use super::*;

    #[test]
    fn clipping_is_optional_and_preserves_the_full_layout_rectangle() {
        let legacy = serde_json::json!({"hwnd": 17, "rect": {"left": 0, "top": -50, "right": 200, "bottom": 50}});
        let mut target: WindowPreviewTarget = serde_json::from_value(legacy.clone()).unwrap();
        assert!(target.clip.is_none());
        assert_eq!(serde_json::to_value(&target).unwrap(), legacy);
        target.clip = Some(Rect {
            left: 0,
            top: 0,
            right: 200,
            bottom: 50,
        });
        let encoded = serde_json::to_value(&target).unwrap();
        assert_eq!(encoded["rect"], legacy["rect"]);
        let decoded: WindowPreviewTarget = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded.clip.unwrap().top, 0);
        assert_eq!(decoded.rect.top, -50);
    }
}
