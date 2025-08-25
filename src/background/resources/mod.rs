pub mod cli;
pub mod commands;
mod emitters;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use seelen_core::{
    resource::{ResourceKind, SluResource},
    state::{IconPack, Plugin, Theme, Wallpaper, Widget},
};

use crate::{
    error::Result,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
};

pub static RESOURCES: LazyLock<Arc<ResourceManager>> =
    LazyLock::new(|| Arc::new(ResourceManager::default()));

#[derive(Default)]
pub struct ResourceManager {
    pub themes: scc::HashMap<PathBuf, Arc<Theme>>,
    pub plugins: scc::HashMap<PathBuf, Arc<Plugin>>,
    pub widgets: scc::HashMap<PathBuf, Arc<Widget>>,
    pub icon_packs: scc::HashMap<PathBuf, Arc<IconPack>>,
    pub wallpapers: scc::HashMap<PathBuf, Arc<Wallpaper>>,
    /// list of manual loaded resources
    pub manual: scc::HashSet<PathBuf>,
}

impl ResourceManager {
    pub fn load(&self, kind: &ResourceKind, path: &Path) -> Result<()> {
        match kind {
            ResourceKind::Theme => {
                let mut theme = Theme::load(path)?;
                if theme.id.starts_with("@deprecated") {
                    return Ok(());
                }
                theme.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_themes_path());
                self.themes.upsert(path.to_path_buf(), Arc::new(theme));
            }
            ResourceKind::Widget => {
                let mut widget = Widget::load(path)?;
                widget.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_widgets_path());
                self.widgets.upsert(path.to_path_buf(), Arc::new(widget));
            }
            ResourceKind::Plugin => {
                let mut plugin = Plugin::load(path)?;
                plugin.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_plugins_path());
                self.plugins.upsert(path.to_path_buf(), Arc::new(plugin));
            }
            ResourceKind::Wallpaper => {
                if path.is_file() {
                    let Some(extension) = path.extension() else {
                        return Err("Wallpaper has no extension".into());
                    };

                    let extension = extension.to_string_lossy().to_lowercase();
                    if Wallpaper::SUPPORTED_IMAGES.contains(&extension.as_ref())
                        || Wallpaper::SUPPORTED_VIDEOS.contains(&extension.as_ref())
                    {
                        let wallpaper = Wallpaper::create_from_file(
                            path,
                            &SEELEN_COMMON
                                .user_wallpapers_path()
                                .join(date_based_hex_id()),
                            // copy if file is outside of user wallpapers (ex: Desktop)
                            !path.starts_with(SEELEN_COMMON.user_wallpapers_path()),
                        )?;
                        self.wallpapers
                            .upsert(path.to_path_buf(), Arc::new(wallpaper));
                    }
                    return Ok(());
                }

                self.wallpapers
                    .upsert(path.to_path_buf(), Arc::new(Wallpaper::load(path)?));
            }
            ResourceKind::IconPack => {
                self.icon_packs
                    .upsert(path.to_path_buf(), Arc::new(IconPack::load(path)?));
            }
            ResourceKind::SoundPack => {
                // feature not implemented
            }
        }
        Ok(())
    }

    pub fn unload(&self, kind: &ResourceKind, path: &Path) {
        match kind {
            ResourceKind::Theme => {
                self.themes.remove(path);
            }
            ResourceKind::Widget => {
                self.widgets.remove(path);
            }
            ResourceKind::Plugin => {
                self.plugins.remove(path);
            }
            ResourceKind::Wallpaper => {
                self.wallpapers.remove(path);
            }
            ResourceKind::IconPack => {
                self.icon_packs.remove(path);
            }
            ResourceKind::SoundPack => {
                // feature not implemented
            }
        }
    }

    pub fn unload_all(&self, kind: &ResourceKind) {
        match kind {
            ResourceKind::Theme => self.themes.retain(|k, _| !self.manual.contains(k)),
            ResourceKind::Plugin => self.plugins.retain(|k, _| !self.manual.contains(k)),
            ResourceKind::Widget => self.widgets.retain(|k, _| !self.manual.contains(k)),
            ResourceKind::IconPack => self.icon_packs.retain(|k, _| !self.manual.contains(k)),
            ResourceKind::Wallpaper => self.wallpapers.retain(|k, _| !self.manual.contains(k)),
            ResourceKind::SoundPack => {
                // feature not implemented
            }
        }
    }

    /// returns a list of dirs to be read by this kind
    fn get_entries_for_type(kind: &ResourceKind) -> Result<Vec<std::fs::ReadDir>> {
        let list = match kind {
            ResourceKind::Theme => {
                let user_path = SEELEN_COMMON.user_themes_path();
                let bundled_path = SEELEN_COMMON.bundled_themes_path();
                vec![
                    std::fs::read_dir(bundled_path)?,
                    std::fs::read_dir(user_path)?,
                ]
            }
            ResourceKind::Widget => {
                let user_path = SEELEN_COMMON.user_widgets_path();
                let bundled_path = SEELEN_COMMON.bundled_widgets_path();
                vec![
                    std::fs::read_dir(bundled_path)?,
                    std::fs::read_dir(user_path)?,
                ]
            }
            ResourceKind::Plugin => {
                let user_path = SEELEN_COMMON.user_plugins_path();
                let bundled_path = SEELEN_COMMON.bundled_plugins_path();
                vec![
                    std::fs::read_dir(bundled_path)?,
                    std::fs::read_dir(user_path)?,
                ]
            }
            ResourceKind::Wallpaper => {
                let user_path = SEELEN_COMMON.user_wallpapers_path();
                vec![std::fs::read_dir(user_path)?]
            }
            ResourceKind::IconPack => {
                let user_path = SEELEN_COMMON.user_icons_path();
                vec![std::fs::read_dir(user_path)?]
            }
            ResourceKind::SoundPack => {
                let user_path = SEELEN_COMMON.user_sounds_path();
                vec![std::fs::read_dir(user_path)?]
            }
        };
        Ok(list)
    }

    pub fn load_all_of_type(&self, kind: ResourceKind) -> Result<()> {
        let entries = Self::get_entries_for_type(&kind)?;
        self.unload_all(&kind);
        for entry in entries.into_iter().flatten().flatten() {
            match self.load(&kind, &entry.path()) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Failed to load {kind:?}, error: {e}");
                }
            }
        }
        Ok(())
    }
}

unsafe impl Send for ResourceManager {}
unsafe impl Sync for ResourceManager {}
