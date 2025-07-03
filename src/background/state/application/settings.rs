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
        trace_lock!(SEELEN).on_settings_change(self)?;
        trace_lock!(WEG_ITEMS_IMPL).emit_to_webview()?;
        Ok(())
    }

    pub(super) fn read_settings(&mut self) -> Result<()> {
        let path_exists = SEELEN_COMMON.settings_path().exists();
        let mut should_write_settings = !path_exists;

        if path_exists {
            self.settings = Self::get_settings_from_path(SEELEN_COMMON.settings_path())?;
            self.settings.migrate()?;
            if !self.settings.old_active_themes.is_empty() {
                should_write_settings = true;
                for theme in self.themes.values() {
                    if self
                        .settings
                        .old_active_themes
                        .contains(&theme.metadata.filename)
                    {
                        self.settings.active_themes.push(theme.id.clone());
                    }
                }
                self.settings.old_active_themes.clear();
            }
            self.settings.sanitize()?;
        }

        if !is_virtual_desktop_supported() {
            self.settings.virtual_desktop_strategy = VirtualDesktopStrategy::Seelen;
        }

        if should_write_settings {
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
