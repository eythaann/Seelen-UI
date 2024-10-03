use std::path::PathBuf;

use itertools::Itertools;
use lazy_static::lazy_static;
use tauri::{path::BaseDirectory, Manager};

use crate::{error_handler::Result, seelen::get_app_handle};

lazy_static! {
    static ref ICONS: Icons = Icons::instance().expect("Failed to load icons paths");

    pub static ref IGNORE_FOCUS: Vec<String> = [
        "Task Switching",
        "Task View",
        "Virtual desktop switching preview",
        "Virtual desktop hotkey switching preview",
        "Seelen Window Manager", // for some reason this sometimes is focused, maybe could be deleted
    ]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();

    /**
     * Some UWP apps like WhatsApp are resized after be opened,
     * this list will be used to resize them back after a delay.
     */
    pub static ref FORCE_RETILING_AFTER_ADD: Vec<String> = ["WhatsApp"]
    .iter()
    .map(|x| x.to_string())
    .collect_vec();
}

pub static NATIVE_UI_POPUP_CLASSES: [&str; 3] = [
    "ForegroundStaging",            // Task Switching and Task View
    "XamlExplorerHostIslandWindow", // Task Switching, Task View and other popups
    "ControlCenterWindow",          // Windows 11 right panel with quick settings
];

pub static OVERLAP_BLACK_LIST_BY_EXE: [&str; 4] = [
    "msedgewebview2.exe",
    "SearchHost.exe",
    "StartMenuExperienceHost.exe",
    "ShellExperienceHost.exe",
];

pub struct Icons {
    missing_app: PathBuf,
}

impl Icons {
    fn instance() -> Result<Self> {
        let handle = get_app_handle();
        Ok(Self {
            missing_app: handle
                .path()
                .resolve("static/icons/missing.png", BaseDirectory::Resource)?,
        })
    }

    pub fn missing_app() -> PathBuf {
        ICONS.missing_app.clone()
    }
}
