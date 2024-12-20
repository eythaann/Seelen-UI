use std::{fs::OpenOptions, io::Write};

use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle};

use super::{FullState, WEG_ITEMS_PATH};

impl FullState {
    pub(super) fn emit_weg_items(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateWegItemsChanged, &self.weg_items)?;
        Ok(())
    }

    pub(super) fn read_weg_items(&mut self) -> Result<()> {
        if WEG_ITEMS_PATH.exists() {
            self.weg_items =
                serde_yaml::from_str(&std::fs::read_to_string(WEG_ITEMS_PATH.as_path())?)?;
            self.weg_items.sanitize();
        } else {
            self.write_weg_items()?;
        }
        Ok(())
    }

    pub fn write_weg_items(&self) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(WEG_ITEMS_PATH.as_path())?;
        file.write_all(serde_yaml::to_string(&self.weg_items)?.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}
