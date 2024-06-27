use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
};

use serde::Deserialize;

use crate::error_handler::Result;

pub static TOOLBAR_HEIGHT: AtomicU32 = AtomicU32::new(30);

#[derive(Debug, Deserialize, Clone)]
pub struct AhkShortcutConfig {
    #[allow(dead_code)]
    pub fancy: String,
    pub ahk: String,
}

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
    #[serde(skip)]
    path: PathBuf,
    /** this is no snake case for a error in naming but is already published so FF */
    seelenweg: Option<FeatureState>,
    seelen_shell: Option<FeatureState>,
    fancy_toolbar: Option<FancyToolbarState>,
    window_manager: Option<FeatureState>,
    ahk_enabled: Option<bool>,
    ahk_variables: Option<HashMap<String, AhkShortcutConfig>>,
}

impl State {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let content = if path.exists() {
            std::fs::read_to_string(path)?
        } else {
            String::from("{}")
        };

        let mut state: Self = serde_json::from_str(&content)?;
        state.path = path.to_path_buf();
        state.store_statics();

        Ok(state)
    }

    pub fn refresh(&mut self) -> Result<()> {
        let path = self.path.clone();
        let content = std::fs::read_to_string(&path)?;

        *self = serde_json::from_str(&content)?;
        self.path = path;
        self.store_statics();

        Ok(())
    }

    pub fn store_statics(&self) {
        if let Some(toolbar) = &self.fancy_toolbar {
            if let Some(height) = toolbar.height {
                TOOLBAR_HEIGHT.store(height, Ordering::SeqCst);
            }
        }
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

    pub fn get_ahk_variables(&self) -> HashMap<String, AhkShortcutConfig> {
        if let Some(variables) = &self.ahk_variables {
            return variables.clone();
        }
        HashMap::new()
    }
}
