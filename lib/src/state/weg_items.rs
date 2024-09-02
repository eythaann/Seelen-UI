use std::{collections::HashSet, path::PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PinnedWegItem {
    /// executable path
    exe: String,
    /// command to open the app using explorer.exe (uwp apps starts with `shell:AppsFolder`)
    execution_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemporalPinnedWegItem {
    /// executable path
    exe: String,
    /// command to open the app using explorer.exe (uwp apps starts with `shell:AppsFolder`)
    execution_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum WegItem {
    PinnedApp(PinnedWegItem),
    TemporalPin(TemporalPinnedWegItem),
    Separator { id: String },
    Media,
    StartMenu,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct WegItems {
    left: Vec<WegItem>,
    center: Vec<WegItem>,
    right: Vec<WegItem>,
}

impl Default for WegItems {
    fn default() -> Self {
        Self {
            left: vec![WegItem::StartMenu],
            center: vec![WegItem::PinnedApp(PinnedWegItem {
                exe: "C:\\Windows\\explorer.exe".to_string(),
                execution_path: "C:\\Windows\\explorer.exe".to_string(),
            })],
            right: vec![WegItem::Media],
        }
    }
}

impl WegItems {
    fn clean_items(dict: &mut HashSet<String>, items: Vec<WegItem>) -> Vec<WegItem> {
        let mut result = Vec::new();
        for item in items {
            match &item {
                WegItem::PinnedApp(app) => {
                    if !dict.contains(&app.exe) {
                        dict.insert(app.exe.clone());
                        // remove apps that don't exist
                        if PathBuf::from(&app.exe).exists() {
                            result.push(item);
                        }
                    }
                }
                WegItem::TemporalPin(app) => {
                    if !dict.contains(&app.exe) {
                        dict.insert(app.exe.clone());
                        // remove apps that don't exist
                        if PathBuf::from(&app.exe).exists() {
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

    pub fn clean_all_items(&mut self) {
        let mut dict = HashSet::new();
        self.left = Self::clean_items(&mut dict, std::mem::take(&mut self.left));
        self.center = Self::clean_items(&mut dict, std::mem::take(&mut self.center));
        self.right = Self::clean_items(&mut dict, std::mem::take(&mut self.right));
    }
}
