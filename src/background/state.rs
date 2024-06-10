use std::path::PathBuf;

use serde::Deserialize;

use crate::error_handler::Result;

#[derive(Debug, Deserialize, Clone)]
struct FeatureState {
    enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
struct FancyToolbarState {
    enabled: Option<bool>,
    height: Option<u32>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct State {
    /** this is no snake case for a error in naming but is already published so FF */
    seelenweg: Option<FeatureState>,
    seelen_shell: Option<FeatureState>,
    fancy_toolbar: Option<FancyToolbarState>,
    window_manager: Option<FeatureState>,
    ahk_enabled: Option<bool>,
}

impl State {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let mut content = String::from("{}");
        if path.exists() {
            content = std::fs::read_to_string(path)?;
        }
        let value: Self = serde_json::from_str(&content)?;
        Ok(value)
    }

    pub fn is_weg_enabled(&self) -> bool {
        if let Some(weg) = &self.seelenweg {
            if let Some(enable) = weg.enabled {
                return enable;
            }
        }
        true
    }

    pub fn is_shell_enabled(&self) -> bool {
        if let Some(shell) = &self.seelen_shell {
            if let Some(enable) = shell.enabled {
                return enable;
            }
        }
        true
    }

    pub fn is_bar_enabled(&self) -> bool {
        if let Some(bar) = &self.fancy_toolbar {
            if let Some(enable) = bar.enabled {
                return enable;
            }
        }
        true
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        if let Some(window_manager) = &self.window_manager {
            if let Some(enable) = window_manager.enabled {
                return enable;
            }
        }
        true
    }

    pub fn is_ahk_enabled(&self) -> bool {
        if let Some(enable) = self.ahk_enabled {
            return enable;
        }
        true
    }

    pub fn get_toolbar_height(&self) -> u32 {
        if let Some(bar) = &self.fancy_toolbar {
            if let Some(height) = bar.height {
                return height;
            }
        }
        30
    }
}
