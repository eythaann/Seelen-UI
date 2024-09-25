use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::rect::Rect;

use super::SeelenWallWallpaper;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct FancyToolbarSettingsByMonitor {
    pub enabled: bool,
}

impl Default for FancyToolbarSettingsByMonitor {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenWegSettingsByMonitor {
    pub enabled: bool,
}

impl Default for SeelenWegSettingsByMonitor {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowManagerSettingsByMonitor {
    pub enabled: bool,
    pub padding: Option<u32>,
    pub margin: Option<Rect>,
    pub gap: Option<u32>,
    pub layout: Option<String>,
}

impl Default for WindowManagerSettingsByMonitor {
    fn default() -> Self {
        Self {
            enabled: true,
            padding: None,
            margin: None,
            gap: None,
            layout: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct SeelenWallSettingsByMonitor {
    pub enabled: bool,
    pub backgrounds: Option<Vec<SeelenWallWallpaper>>,
}

impl Default for SeelenWallSettingsByMonitor {
    fn default() -> Self {
        Self {
            enabled: true,
            backgrounds: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum WorkspaceIdentifierType {
    #[serde(alias = "name")]
    Name,
    #[serde(alias = "index")]
    Index,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceIdentifier {
    pub id: String,
    pub kind: WorkspaceIdentifierType,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfiguration {
    pub identifier: WorkspaceIdentifier,
    pub layout: Option<String>,
    pub backgrounds: Option<Vec<SeelenWallWallpaper>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase")]
pub struct MonitorConfiguration {
    pub tb: FancyToolbarSettingsByMonitor,
    pub weg: SeelenWegSettingsByMonitor,
    pub wm: WindowManagerSettingsByMonitor,
    pub wall: SeelenWallSettingsByMonitor,
    /// list of settings by workspace on this monitor
    pub workspaces_v2: Vec<WorkspaceConfiguration>,
}
