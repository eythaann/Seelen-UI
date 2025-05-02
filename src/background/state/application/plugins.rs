use std::path::Path;

use itertools::Itertools;
use seelen_core::{
    handlers::SeelenEvent,
    resource::{ConcreteResource, SluResourceFile},
    state::Plugin,
};
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    pub(super) fn emit_plugins(&self) -> Result<()> {
        get_app_handle().emit(
            SeelenEvent::StatePluginsChanged,
            &self.plugins.values().collect_vec(),
        )?;
        Ok(())
    }

    fn load_plugin_from_file(path: &Path) -> Result<Plugin> {
        let ext = path
            .extension()
            .ok_or("Invalid file extension")?
            .to_string_lossy();

        let plugin = match ext.as_ref() {
            "yaml" | "yml" => serde_yaml::from_slice(&std::fs::read(path)?)?,
            "slu" => {
                let file = SluResourceFile::load(path)?;
                match file.concrete()? {
                    ConcreteResource::Plugin(plugin) => plugin,
                    _ => return Err("Resource file is not a plugin".into()),
                }
            }
            _ => return Err("Invalid file extension".into()),
        };
        Ok(plugin)
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
            match Self::load_plugin_from_file(&path) {
                Ok(mut plugin) => {
                    plugin.metadata.bundled = path.starts_with(bundled_path);
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
