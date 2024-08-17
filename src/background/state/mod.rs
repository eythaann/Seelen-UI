pub mod application;
pub mod domain;
pub mod infrastructure;

use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, AtomicU32},
};

use application::FullState;
use domain::AhkVar;

pub static IS_TOOLBAR_ENABLED: AtomicBool = AtomicBool::new(true);
pub static TOOLBAR_HEIGHT: AtomicU32 = AtomicU32::new(30);

impl FullState {
    pub fn is_weg_enabled(&self) -> bool {
        self.settings().seelenweg.enabled
    }

    pub fn is_shell_enabled(&self) -> bool {
        false
    }

    pub fn is_bar_enabled(&self) -> bool {
        self.settings().fancy_toolbar.enabled
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        self.settings().window_manager.enabled
    }

    pub fn is_ahk_enabled(&self) -> bool {
        self.settings().ahk_enabled
    }

    pub fn get_ahk_variables(&self) -> HashMap<String, AhkVar> {
        let json_value = serde_json::to_value(&self.settings().ahk_variables)
            .expect("Failed to serialize AHK variables");
        serde_json::from_value(json_value).expect("Failed to deserialize AHK variables")
    }
}
