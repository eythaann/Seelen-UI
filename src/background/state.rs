use std::path::PathBuf;

use serde::Deserialize;

use crate::error_handler::Result;

#[derive(Debug, Deserialize)]
struct SeelenSwegState {
    enable: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct State {
    seelen_weg: Option<SeelenSwegState>,
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
}