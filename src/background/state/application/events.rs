use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{app::get_app_handle, error::Result};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings_by_app(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateSettingsByAppChanged,
            &self.settings_by_app,
        )?;
        Ok(())
    }

    pub(super) fn emit_history(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateHistoryChanged, &self.launcher_history)?;
        Ok(())
    }
}
