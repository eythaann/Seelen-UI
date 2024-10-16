use itertools::Itertools;
use seelen_core::{handlers::SeelenEvent, state::WegItems};
use tauri::Emitter;

use crate::{
    error_handler::Result,
    seelen::{get_app_handle, SEELEN},
    trace_lock,
};

use super::FullState;

impl FullState {
    pub(super) fn emit_settings(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateSettingsChanged, self.settings())?;
        trace_lock!(SEELEN).on_settings_change()?;
        Ok(())
    }

    pub fn emit_weg_items(&self, items: &WegItems) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateWegItemsChanged, items)?;
        Ok(())
    }

    pub(super) fn emit_themes(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateThemesChanged,
            self.themes().values().collect_vec(),
        )?;
        Ok(())
    }

    pub(super) fn emit_placeholders(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StatePlaceholdersChanged,
            self.placeholders().values().collect_vec(),
        )?;
        Ok(())
    }

    pub(super) fn emit_layouts(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateLayoutsChanged,
            self.layouts().values().collect_vec(),
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
        get_app_handle().emit(SeelenEvent::StateHistoryChanged, self.history())?;
        Ok(())
    }

    pub(super) fn emit_icon_packs(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateIconPacksChanged,
            trace_lock!(self.icon_packs()).values().collect_vec(),
        )?;
        Ok(())
    }
}
