use seelen_core::{handlers::SeelenEvent, state::Settings};
use tauri::Emitter;

use crate::{
    app::{get_app_handle, SEELEN},
    error::Result,
    trace_lock,
    utils::constants::SEELEN_COMMON,
    widgets::weg::weg_items_impl::SEELEN_WEG_STATE,
};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateSettingsChanged, &self.settings)?;
        trace_lock!(SEELEN).on_settings_change(self)?;
        trace_lock!(SEELEN_WEG_STATE).emit_to_webview()?;
        Ok(())
    }

    fn _read_settings(&mut self) -> Result<()> {
        let path = SEELEN_COMMON.settings_path();
        if path.exists() {
            self.settings = Settings::load(path)?;
        } else {
            self.write_settings()?; // create initial settings file
        }
        Ok(())
    }

    pub(super) fn read_settings(&mut self) {
        if let Err(err) = self._read_settings() {
            log::error!("Failed to read settings: {err}");
            Self::show_corrupted_state_to_user(SEELEN_COMMON.settings_path());
        }
    }

    pub fn write_settings(&self) -> Result<()> {
        self.settings.save(SEELEN_COMMON.settings_path())?;
        Ok(())
    }
}
