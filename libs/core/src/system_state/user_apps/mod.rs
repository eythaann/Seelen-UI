use std::path::PathBuf;

use super::Color;
use crate::{rect::Rect, system_state::MonitorId};

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
