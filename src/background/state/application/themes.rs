use itertools::Itertools;
use seelen_core::{handlers::SeelenEvent, state::Theme};
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    pub(super) fn emit_themes(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateThemesChanged,
            self.themes().values().collect_vec(),
        )?;
        Ok(())
    }

    pub(super) fn load_themes(&mut self) -> Result<()> {
        self.themes.clear();
        let entries = std::fs::read_dir(SEELEN_COMMON.bundled_themes_path())?
            .chain(std::fs::read_dir(SEELEN_COMMON.user_themes_path())?);
        for entry in entries.flatten() {
            let path = entry.path();
            match Theme::load(&path) {
                Ok(mut theme) => {
                    theme.metadata.filename = entry.file_name().to_string_lossy().to_string();
                    self.themes.insert(theme.metadata.filename.clone(), theme);
                }
                Err(err) => log::error!("Failed to load theme ({:?}): {:?}", entry.path(), err),
            }
        }
        Ok(())
    }
}
