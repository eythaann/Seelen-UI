use std::path::PathBuf;

use serde::Deserialize;

use crate::error_handler::Result;

#[derive(Debug, Deserialize)]
struct FeatureState {
    enable: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct State {
    seelen_weg: Option<FeatureState>,
    seelen_shell: Option<FeatureState>,
    seelen_bar: Option<FeatureState>,
    seelen_window_manager: Option<FeatureState>,
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
        if let Some(weg) = &self.seelen_weg {
            if let Some(enable) = weg.enable {
                return enable;
            }
        }
        return true;
    }

    pub fn is_shell_enabled(&self) -> bool {
        if let Some(shell) = &self.seelen_shell {
            if let Some(enable) = shell.enable {
                return enable;
            }
        }
        return true;
    }

    pub fn is_bar_enabled(&self) -> bool {
        if let Some(bar) = &self.seelen_bar {
            if let Some(enable) = bar.enable {
                return enable;
            }
        }
        return true;
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        if let Some(window_manager) = &self.seelen_window_manager {
            if let Some(enable) = window_manager.enable {
                return enable;
            }
        }
        return true;
    }
}