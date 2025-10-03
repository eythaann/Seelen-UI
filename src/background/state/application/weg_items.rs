use std::{fs::OpenOptions, io::Write};

use seelen_core::{
    handlers::SeelenEvent,
    state::{WegItem, WegItems},
};
use tauri::Emitter;

use crate::{
    app::get_app_handle, error::Result, modules::uwp::UwpManager, trace_lock,
    utils::constants::SEELEN_COMMON, widgets::weg::weg_items_impl::SEELEN_WEG_STATE,
};

use super::FullState;

impl FullState {
    fn update_weg_items_paths(items: &mut [WegItem]) {
        for item in items {
            match item {
                WegItem::Pinned(data) | WegItem::Temporal(data) => {
                    if let Some(umid) = &data.umid {
                        if let Ok(Some(app_path)) = UwpManager::get_app_path(umid) {
                            data.path = app_path;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn emit_weg_items(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateWegItemsChanged, &self.weg_items)?;
        trace_lock!(SEELEN_WEG_STATE).on_stored_changed(self.weg_items.clone())?;
        Ok(())
    }

    fn _read_weg_items(&mut self) -> Result<()> {
        if SEELEN_COMMON.weg_items_path().exists() {
            self.weg_items =
                serde_yaml::from_str(&std::fs::read_to_string(SEELEN_COMMON.weg_items_path())?)?;
            self.weg_items.sanitize();
            Self::update_weg_items_paths(&mut self.weg_items.left);
            Self::update_weg_items_paths(&mut self.weg_items.center);
            Self::update_weg_items_paths(&mut self.weg_items.right);
        } else {
            self.weg_items.sanitize();
            self.write_weg_items(&self.weg_items)?;
        }
        Ok(())
    }

    pub(super) fn read_weg_items(&mut self) {
        if let Err(err) = self._read_weg_items() {
            log::error!("Failed to read weg items: {err}");
            Self::show_corrupted_state_to_user(SEELEN_COMMON.weg_items_path());
        }
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
