use std::path::Path;

use seelen_core::{handlers::SeelenEvent, state::Plugin};
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    pub(super) fn emit_plugins(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StatePluginsChanged, &self.plugins)?;
        Ok(())
    }

    fn load_plugin_from_file(path: &Path) -> Result<Plugin> {
        Ok(serde_yaml::from_str(&std::fs::read_to_string(path)?)?)
    }

    pub(super) fn load_plugins(&mut self) -> Result<()> {
        let user_path = SEELEN_COMMON.user_plugins_path();
        let bundled_path = SEELEN_COMMON.bundled_plugins_path();

        let entries = std::fs::read_dir(bundled_path)?.chain(std::fs::read_dir(user_path)?);
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            match Self::load_plugin_from_file(&path) {
                Ok(mut plugin) => {
                    plugin.bundled = path.starts_with(bundled_path);
                    self.plugins.insert(plugin.id.clone(), plugin);
                }
                Err(e) => {
                    log::error!("Failed to load plugin: {}", e);
                }
            }
        }
        Ok(())
    }
}
