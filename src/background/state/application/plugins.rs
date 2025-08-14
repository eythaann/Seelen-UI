use itertools::Itertools;
use seelen_core::{handlers::SeelenEvent, resource::SluResource, state::Plugin};
use tauri::Emitter;

use crate::{app::get_app_handle, error::Result, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    pub(super) fn emit_plugins(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StatePluginsChanged,
            &self.plugins.values().collect_vec(),
        )?;
        Ok(())
    }

    pub(super) fn load_plugins(&mut self) -> Result<()> {
        let user_path = SEELEN_COMMON.user_plugins_path();
        let bundled_path = SEELEN_COMMON.bundled_plugins_path();
        self.plugins.clear();

        let entries = std::fs::read_dir(bundled_path)?.chain(std::fs::read_dir(user_path)?);
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            match Plugin::load(&path) {
                Ok(mut plugin) => {
                    plugin.metadata.bundled = path.starts_with(bundled_path);
                    self.plugins.insert(plugin.metadata.path.clone(), plugin);
                }
                Err(e) => {
                    log::error!("Failed to load plugin: {e}");
                }
            }
        }
        Ok(())
    }
}
