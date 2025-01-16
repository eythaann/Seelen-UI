use std::{fs::OpenOptions, io::Write};

use seelen_core::{handlers::SeelenEvent, state::WegItems};
use tauri::Emitter;

use crate::{
    error_handler::Result, seelen::get_app_handle, seelen_weg::weg_items_impl::WEG_ITEMS_IMPL,
    trace_lock, utils::constants::SEELEN_COMMON,
};

use super::FullState;

impl FullState {
    pub fn emit_weg_items(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateWegItemsChanged, &self.weg_items)?;
        trace_lock!(WEG_ITEMS_IMPL).on_stored_changed(self.weg_items.clone())?;
        Ok(())
    }

    pub(super) fn read_weg_items(&mut self) -> Result<()> {
        if SEELEN_COMMON.weg_items_path().exists() {
            self.weg_items =
                serde_yaml::from_str(&std::fs::read_to_string(SEELEN_COMMON.weg_items_path())?)?;
            self.weg_items.sanitize();
        } else {
            self.write_weg_items(&self.weg_items)?;
        }
        Ok(())
    }

    pub fn write_weg_items(&self, items: &WegItems) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(SEELEN_COMMON.weg_items_path())?;
        file.write_all(serde_yaml::to_string(items)?.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}
