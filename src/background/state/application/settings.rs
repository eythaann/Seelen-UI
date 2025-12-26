use seelen_core::{handlers::SeelenEvent, state::Settings};

use crate::{
    app::{emit_to_webviews, SEELEN},
    error::Result,
    resources::RESOURCES,
    trace_lock,
    utils::constants::SEELEN_COMMON,
    widgets::weg::weg_items_impl::SEELEN_WEG_STATE,
};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings(&self) -> Result<()> {
        emit_to_webviews(SeelenEvent::StateSettingsChanged, &self.settings);

        trace_lock!(SEELEN).on_settings_change(self)?;
        trace_lock!(SEELEN_WEG_STATE).emit_to_webview()?;
        Ok(())
    }

    fn _read_settings(&mut self) -> Result<()> {
        let path = SEELEN_COMMON.settings_path();
        if path.exists() {
            self.settings = Settings::load(path)?;
            self.sanitize_wallpaper_collections();
        } else {
            self.write_settings()?; // create initial settings file
        }
        Ok(())
    }

    /// Sanitize wallpaper collections to remove non-existent wallpaper IDs
    pub(super) fn sanitize_wallpaper_collections(&mut self) -> bool {
        let mut changed = false;
        for collection in &mut self.settings.wallpaper_collections {
            let original_len = collection.wallpapers.len();
            collection
                .wallpapers
                .retain(|wallpaper_id| RESOURCES.wallpapers.contains(wallpaper_id));
            if collection.wallpapers.len() != original_len {
                changed = true;
            }
        }
        changed
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
