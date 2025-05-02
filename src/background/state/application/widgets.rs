use std::path::PathBuf;

use seelen_core::{handlers::SeelenEvent, state::Widget};
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    pub(super) fn emit_widgets(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateWidgetsChanged, &self.plugins)?;
        Ok(())
    }

    fn load_widget_from_file(path: PathBuf) -> Result<Widget> {
        Ok(serde_yaml::from_str(&std::fs::read_to_string(&path)?)?)
    }

    fn load_widget_from_folder(path: PathBuf) -> Result<Widget> {
        let mut widget = Self::load_widget_from_file(path.join("metadata.yml"))?;
        widget.js = Some(std::fs::read_to_string(path.join("index.js"))?);
        widget.css = Some(std::fs::read_to_string(path.join("index.css"))?);
        widget.html = Some(std::fs::read_to_string(path.join("index.html"))?);
        Ok(widget)
    }

    pub(super) fn load_widgets(&mut self) -> Result<()> {
        let user_path = SEELEN_COMMON.user_widgets_path();
        let bundled_path = SEELEN_COMMON.bundled_widgets_path();
        self.widgets.clear();

        let entries = std::fs::read_dir(bundled_path)?.chain(std::fs::read_dir(user_path)?);
        for entry in entries.flatten() {
            let path = entry.path();
            let widget = if path.is_dir() {
                Self::load_widget_from_folder(path)
            } else {
                Self::load_widget_from_file(path)
            };
            match widget {
                Ok(widget) => {
                    self.widgets.insert(widget.id.clone(), widget);
                }
                Err(e) => {
                    log::error!("Failed to load widget: {}", e);
                }
            }
        }
        Ok(())
    }
}
