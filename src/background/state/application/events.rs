use itertools::Itertools;
use tauri::Emitter;

use crate::{error_handler::Result, seelen::SEELEN, trace_lock};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings(&self) -> Result<()> {
        self.handle.emit("settings-changed", self.settings())?;
        trace_lock!(SEELEN).on_settings_change()?;
        Ok(())
    }

    pub(super) fn emit_weg_items(&self) -> Result<()> {
        self.handle.emit("weg-items", self.weg_items())?;
        Ok(())
    }

    pub(super) fn emit_themes(&self) -> Result<()> {
        self.handle
            .emit("themes", self.themes().values().collect_vec())?;
        Ok(())
    }

    pub(super) fn emit_placeholders(&self) -> Result<()> {
        self.handle
            .emit("placeholders", self.placeholders().values().collect_vec())?;
        Ok(())
    }

    pub(super) fn emit_layouts(&self) -> Result<()> {
        self.handle
            .emit("layouts", self.layouts().values().collect_vec())?;
        Ok(())
    }

    pub(super) fn emit_settings_by_app(&self) -> Result<()> {
        self.handle
            .emit("settings-by-app", self.settings_by_app())?;
        Ok(())
    }

    pub(super) fn emit_history(&self) -> Result<()> {
        self.handle.emit("history", self.history())?;
        Ok(())
    }
}
