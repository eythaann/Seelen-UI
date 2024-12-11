use std::{fs::OpenOptions, io::Write};

use seelen_core::{handlers::SeelenEvent, state::VirtualDesktopStrategy};
use tauri::Emitter;

use crate::{
    error_handler::Result,
    seelen::{get_app_handle, SEELEN},
    trace_lock,
    utils::is_virtual_desktop_supported,
};

use super::{FullState, USER_SETTINGS_PATH};

impl FullState {
    pub(super) fn emit_settings(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateSettingsChanged, self.settings())?;
        trace_lock!(SEELEN).on_settings_change()?;
        Ok(())
    }

    pub(super) fn read_settings(&mut self) -> Result<()> {
        let path_exists = USER_SETTINGS_PATH.exists();
        if path_exists {
            self.settings = Self::get_settings_from_path(&USER_SETTINGS_PATH)?;
            self.settings.sanitize();
        }
        if !is_virtual_desktop_supported() {
            self.settings.virtual_desktop_strategy = VirtualDesktopStrategy::Seelen;
        }
        // create settings file
        if !path_exists {
            self.write_settings()?;
        }
        Ok(())
    }

    pub fn write_settings(&self) -> Result<()> {
        let mut temp_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(USER_SETTINGS_PATH.as_path())?;
        temp_file.write_all(serde_json::to_string_pretty(&self.settings)?.as_bytes())?;
        temp_file.flush()?;
        Ok(())
    }
}
