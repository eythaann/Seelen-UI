use itertools::Itertools;
use seelen_core::{handlers::SeelenEvent, resource::SluResource, state::Theme};
use tauri::Emitter;

use crate::{app::get_app_handle, error::Result, utils::constants::SEELEN_COMMON};

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
                    if theme.id.starts_with("@deprecated") {
                        continue;
                    }
                    theme.metadata.bundled = path.starts_with(SEELEN_COMMON.bundled_themes_path());
                    self.themes.insert(theme.metadata.path.clone(), theme);
                }
                Err(err) => log::error!("Failed to load theme ({:?}): {:?}", entry.path(), err),
            }
        }
        Ok(())
    }
}
