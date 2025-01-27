use itertools::Itertools;
use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle};

use super::FullState;

impl FullState {
    pub(super) fn emit_themes(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateThemesChanged,
            self.themes().values().collect_vec(),
        )?;
        Ok(())
    }

    pub(super) fn emit_settings_by_app(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateSettingsByAppChanged,
            self.settings_by_app(),
        )?;
        Ok(())
    }

    pub(super) fn emit_history(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateHistoryChanged, self.launcher_history())?;
        Ok(())
    }
}
