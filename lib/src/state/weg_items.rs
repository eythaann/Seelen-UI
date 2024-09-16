use std::{collections::HashSet, path::PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum WegItem {
    Pinned {
        path: PathBuf,
        is_dir: bool,
    },
    PinnedApp {
        exe: PathBuf,
        /// command to open the app using explorer.exe (UWP apps start with `shell:AppsFolder`)
        execution_path: String,
    },
    TemporalPin {
        exe: PathBuf,
        /// command to open the app using explorer.exe (UWP apps start with `shell:AppsFolder`)
        execution_path: String,
    },
    Separator {
        id: String,
    },
    Media,
    StartMenu,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct WegItems {
    pub left: Vec<WegItem>,
    pub center: Vec<WegItem>,
    pub right: Vec<WegItem>,
}

impl Default for WegItems {
    fn default() -> Self {
        Self {
            left: vec![WegItem::StartMenu],
            center: vec![WegItem::PinnedApp {
                exe: "C:\\Windows\\explorer.exe".into(),
                execution_path: "C:\\Windows\\explorer.exe".into(),
            }],
            right: vec![WegItem::Media],
        }
    }
}

impl WegItems {
    fn sanitize_items(dict: &mut HashSet<String>, items: Vec<WegItem>) -> Vec<WegItem> {
        let mut result = Vec::new();
        for item in items {
            match &item {
                WegItem::Pinned { path, is_dir: _ } => {
                    let file = path.to_string_lossy().to_string();
                    if !dict.contains(&file) {
                        dict.insert(file);
                        result.push(item);
                    }
                }
                WegItem::PinnedApp {
                    exe,
                    execution_path: _,
                } => {
                    let exe = exe.to_string_lossy().to_string();
                    if !dict.contains(&exe) {
                        dict.insert(exe.clone());
                        // remove apps that don't exist
                        if PathBuf::from(&exe).exists() {
                            result.push(item);
                        }
                    }
                }
                WegItem::TemporalPin {
                    exe,
                    execution_path: _,
                } => {
                    let exe = exe.to_string_lossy().to_string();
                    if !dict.contains(&exe) {
                        dict.insert(exe.clone());
                        // remove apps that don't exist
                        if PathBuf::from(&exe).exists() {
                            result.push(item);
                        }
                    }
                }
                WegItem::Separator { id } => {
                    if !dict.contains(id) {
                        dict.insert(id.clone());
                        result.push(item);
                    }
                }
                WegItem::StartMenu => {
                    if !dict.contains("StartMenu") {
                        result.push(item);
                        dict.insert("StartMenu".to_owned());
                    }
                }
                WegItem::Media => {
                    if !dict.contains("Media") {
                        result.push(item);
                        dict.insert("Media".to_owned());
                    }
                }
            }
        }
        result
    }

    pub fn sanitize(&mut self) {
        let mut dict = HashSet::new();
        self.left = Self::sanitize_items(&mut dict, std::mem::take(&mut self.left));
        self.center = Self::sanitize_items(&mut dict, std::mem::take(&mut self.center));
        self.right = Self::sanitize_items(&mut dict, std::mem::take(&mut self.right));
    }
}
