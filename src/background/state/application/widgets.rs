use itertools::Itertools;
use seelen_core::{handlers::SeelenEvent, state::Widget};
use tauri::Emitter;

use crate::{
    error_handler::Result,
    seelen::{get_app_handle, SEELEN},
    trace_lock,
    utils::constants::SEELEN_COMMON,
};

use super::FullState;

impl FullState {
    pub(super) fn emit_widgets(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StateWidgetsChanged,
            &self.widgets.values().collect_vec(),
        )?;
        trace_lock!(SEELEN).on_widgets_change(self)?;
        Ok(())
    }

    pub(super) fn load_widgets(&mut self) -> Result<()> {
        let user_path = SEELEN_COMMON.user_widgets_path();
        let bundled_path = SEELEN_COMMON.bundled_widgets_path();
        self.widgets.clear();

        let entries = std::fs::read_dir(bundled_path)?.chain(std::fs::read_dir(user_path)?);
        for entry in entries.flatten() {
            let path = entry.path();
            match Widget::load(&path) {
                Ok(mut widget) => {
                    widget.metadata.bundled = path.starts_with(bundled_path);
                    widget.metadata.filename = entry.file_name().to_string_lossy().to_string();
                    self.widgets.insert(widget.id.clone(), widget);
                }
                Err(e) => {
                    log::error!("Failed to load widget: {e}");
                }
            }
        }
        Ok(())
    }
}
