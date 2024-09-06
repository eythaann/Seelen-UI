pub mod application;
pub mod domain;
pub mod infrastructure;

use std::collections::HashMap;

use application::FullState;
use domain::AhkVar;

impl FullState {
    pub fn is_weg_enabled(&self) -> bool {
        self.settings().seelenweg.enabled
    }

    pub fn is_bar_enabled(&self) -> bool {
        self.settings().fancy_toolbar.enabled
    }

    pub fn is_window_manager_enabled(&self) -> bool {
        self.settings().window_manager.enabled
    }

    pub fn is_rofi_enabled(&self) -> bool {
        true
    }

    pub fn is_ahk_enabled(&self) -> bool {
        self.settings().ahk_enabled
    }

    pub fn get_ahk_variables(&self) -> HashMap<String, AhkVar> {
        self.settings().ahk_variables.as_hash_map()
    }
}
