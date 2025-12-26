use seelen_core::handlers::SeelenEvent;

use crate::{app::emit_to_webviews, error::Result};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings_by_app(&self) -> Result<()> {
        emit_to_webviews(
            SeelenEvent::StateSettingsByAppChanged,
            self.settings_by_app.as_slice(),
        );
        Ok(())
    }

    pub(super) fn emit_history(&self) -> Result<()> {
        emit_to_webviews(SeelenEvent::StateHistoryChanged, &self.launcher_history);
        Ok(())
    }
}
