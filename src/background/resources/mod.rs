pub mod cli;
pub mod commands;
mod emitters;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use seelen_core::{
    resource::{IconPackId, PluginId, ResourceKind, SluResource, ThemeId, WallpaperId, WidgetId},
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
    pub themes: scc::HashMap<ThemeId, Arc<Theme>>,
    pub plugins: scc::HashMap<PluginId, Arc<Plugin>>,
    pub widgets: scc::HashMap<WidgetId, Arc<Widget>>,
    pub icon_packs: scc::HashMap<IconPackId, Arc<IconPack>>,
    pub wallpapers: scc::HashMap<WallpaperId, Arc<Wallpaper>>,
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
                self.themes.upsert(theme.id.clone(), Arc::new(theme));
            }
            ResourceKind::Widget => {
                let mut widget = Widget::load(path)?;
                widget.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_widgets_path());

                widget
                    .plugins
                    .retain(|plugin| !plugin.metadata.internal.path.starts_with(path));

                for mut plugin in widget.plugins.clone() {
                    plugin.metadata.internal = widget.metadata.internal.clone();
                    self.plugins.upsert(plugin.id.clone(), Arc::new(plugin));
                }

                self.widgets.upsert(widget.id.clone(), Arc::new(widget));
            }
            ResourceKind::Plugin => {
                let mut plugin = Plugin::load(path)?;
                plugin.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_plugins_path());
                self.plugins.upsert(plugin.id.clone(), Arc::new(plugin));
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
                            .upsert(wallpaper.id.clone(), Arc::new(wallpaper));
                    }
                    return Ok(());
                }

                let wallpaper = Wallpaper::load(path)?;
                self.wallpapers
                    .upsert(wallpaper.id.clone(), Arc::new(wallpaper));
            }
            ResourceKind::IconPack => {
                let mut icon_pack = IconPack::load(path)?;
                icon_pack.metadata.internal.bundled =
                    path == SEELEN_COMMON.user_icons_path().join("system");
                self.icon_packs
                    .upsert(icon_pack.id.clone(), Arc::new(icon_pack));
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
                self.themes.retain(|_, v| v.metadata.internal.path != path);
            }
            ResourceKind::Widget => {
                self.widgets.retain(|_, v| v.metadata.internal.path != path);
            }
            ResourceKind::Plugin => {
                self.plugins.retain(|_, v| v.metadata.internal.path != path);
            }
            ResourceKind::Wallpaper => {
                self.wallpapers
                    .retain(|_, v| v.metadata.internal.path != path);
            }
            ResourceKind::IconPack => {
                self.icon_packs
                    .retain(|_, v| v.metadata.internal.path != path);
            }
            ResourceKind::SoundPack => {
                // feature not implemented
            }
        }
    }

    pub fn unload_all(&self, kind: &ResourceKind) {
        match kind {
            ResourceKind::Theme => self
                .themes
                .retain(|_, v| !self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::Plugin => self
                .plugins
                .retain(|_, v| !self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::Widget => self
                .widgets
                .retain(|_, v| !self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::IconPack => self
                .icon_packs
                .retain(|_, v| !self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::Wallpaper => self
                .wallpapers
                .retain(|_, v| !self.manual.contains(&v.metadata.internal.path)),
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
