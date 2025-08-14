use std::{fs::OpenOptions, io::Write, path::Path};

use seelen_core::{handlers::SeelenEvent, state::Settings};
use tauri::Emitter;

use crate::{
    app::{get_app_handle, SEELEN},
    error::Result,
    trace_lock,
    utils::constants::SEELEN_COMMON,
    widgets::weg::weg_items_impl::WEG_ITEMS_IMPL,
};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateSettingsChanged, self.settings())?;
        trace_lock!(SEELEN).on_settings_change(self)?;
        trace_lock!(WEG_ITEMS_IMPL).emit_to_webview()?;
        Ok(())
    }

    pub fn get_settings_from_path(path: &Path) -> Result<Settings> {
        match path.extension() {
            Some(ext) if ext == "json" => {
                Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
            }
            _ => Err("Invalid settings file extension".into()),
        }
    }

    fn _read_settings(&mut self) -> Result<()> {
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

        if should_write_settings {
            self.write_settings()?;
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
