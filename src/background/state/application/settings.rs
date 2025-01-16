use std::{fs::OpenOptions, io::Write};

use seelen_core::{handlers::SeelenEvent, state::VirtualDesktopStrategy};
use tauri::Emitter;

use crate::{
    error_handler::Result,
    seelen::{get_app_handle, SEELEN},
    seelen_weg::weg_items_impl::WEG_ITEMS_IMPL,
    trace_lock,
    utils::{constants::SEELEN_COMMON, is_virtual_desktop_supported},
};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateSettingsChanged, self.settings())?;
        trace_lock!(SEELEN).on_settings_change()?;
        trace_lock!(WEG_ITEMS_IMPL).emit_to_webview()?;
        Ok(())
    }

    pub(super) fn read_settings(&mut self) -> Result<()> {
        let path_exists = SEELEN_COMMON.settings_path().exists();
        if path_exists {
            self.settings = Self::get_settings_from_path(SEELEN_COMMON.settings_path())?;
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
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(SEELEN_COMMON.settings_path())?;
        file.write_all(serde_json::to_string_pretty(&self.settings)?.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}
