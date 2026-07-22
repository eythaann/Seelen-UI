pub mod cli;
pub mod commands;
mod emitters;
mod system_icon_pack;
pub mod user_icon_pack;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use seelen_core::{
    handlers::SeelenEvent,
    resource::{
        IconPackId, PluginId, ResourceId, ResourceKind, SluResource, ThemeId, WallpaperId, WidgetId,
    },
    state::{IconPack, Plugin, Theme, Wallpaper, WallpaperCollection, Widget},
};
use uuid::Uuid;

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    state::application::FULL_STATE,
    utils::{constants::SEELEN_COMMON, date_based_hex_id, lock_free::TracedMutex},
};

pub static RESOURCES: LazyLock<Arc<ResourceManager>> =
    LazyLock::new(|| Arc::new(ResourceManager::default()));

#[derive(Default)]
pub struct ResourceManager {
    pub themes: scc::HashMap<ThemeId, Arc<Theme>>,
    pub plugins: scc::HashMap<PluginId, Arc<Plugin>>,
    pub widgets: scc::HashMap<WidgetId, Arc<Widget>>,
    pub wallpapers: scc::HashMap<WallpaperId, Arc<Wallpaper>>,
    /// this list doesn't include the system icon pack, this is managed by the system_icon_pack field.
    pub icon_packs: scc::HashMap<IconPackId, Arc<IconPack>>,
    /// current system icon pack
    pub system_icon_pack: Arc<TracedMutex<Option<IconPack>>>,
    /// list of manual loaded resources
    pub manual: scc::HashSet<PathBuf>,
}

impl ResourceManager {
    pub async fn initialize(&self) {
        tokio::join!(
            async {
                let t = std::time::Instant::now();
                self.load_all_of_type(ResourceKind::Theme).await.log_error();
                log::info!("Themes loaded in {:?}", t.elapsed());
            },
            async {
                let t = std::time::Instant::now();
                self.load_all_of_type(ResourceKind::Plugin)
                    .await
                    .log_error();
                log::info!("Plugins loaded in {:?}", t.elapsed());
            },
            async {
                let t = std::time::Instant::now();
                self.load_all_of_type(ResourceKind::Widget)
                    .await
                    .log_error();
                log::info!("Widgets loaded in {:?}", t.elapsed());
            },
            async {
                let t = std::time::Instant::now();
                self.load_all_of_type(ResourceKind::Wallpaper)
                    .await
                    .log_error();
                log::info!("Wallpapers loaded in {:?}", t.elapsed());
            },
            async {
                let t = std::time::Instant::now();
                self.load_all_of_type(ResourceKind::IconPack)
                    .await
                    .log_error();
                log::info!("IconPacks loaded in {:?}", t.elapsed());
            },
        );
    }

    /// Returns the id of the resource that was loaded, if any (e.g. a deprecated theme
    /// or the system icon pack are loaded but don't produce an activatable resource id).
    pub async fn load(&self, kind: &ResourceKind, path: &Path) -> Result<Option<ResourceId>> {
        let id = match kind {
            ResourceKind::Theme => {
                let mut theme = Theme::load(path).await?;
                if theme.id.starts_with("@deprecated") {
                    return Ok(None);
                }
                theme.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_themes_path());
                let id = (*theme.id).clone();
                self.themes.upsert(theme.id.clone(), Arc::new(theme));
                id
            }
            ResourceKind::Widget => {
                let mut widget = Widget::load(path).await?;
                widget.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_widgets_path());

                widget
                    .plugins
                    .retain(|plugin| !plugin.metadata.internal.path.starts_with(path));

                for mut plugin in widget.plugins.clone() {
                    plugin.metadata.internal = widget.metadata.internal.clone();
                    // plugins inherit the parent widget's premium flag
                    plugin.metadata.premium |= widget.metadata.premium;
                    self.plugins.upsert(plugin.id.clone(), Arc::new(plugin));
                }

                let id = (*widget.id).clone();
                self.widgets.upsert(widget.id.clone(), Arc::new(widget));
                id
            }
            ResourceKind::Plugin => {
                let mut plugin = Plugin::load(path).await?;
                plugin.metadata.internal.bundled =
                    path.starts_with(SEELEN_COMMON.bundled_plugins_path());
                let id = (*plugin.id).clone();
                self.plugins.upsert(plugin.id.clone(), Arc::new(plugin));
                id
            }
            ResourceKind::Wallpaper => {
                let path_is_file = tokio::fs::metadata(path)
                    .await
                    .map(|m| m.is_file())
                    .unwrap_or(false);
                if path_is_file {
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
                        )
                        .await?;
                        let id = (*wallpaper.id).clone();
                        self.wallpapers
                            .upsert(wallpaper.id.clone(), Arc::new(wallpaper));
                        return Ok(Some(id));
                    }
                    return Ok(None);
                }

                let wallpaper = Wallpaper::load(path).await?;
                let id = (*wallpaper.id).clone();
                self.wallpapers
                    .upsert(wallpaper.id.clone(), Arc::new(wallpaper));
                id
            }
            ResourceKind::IconPack => {
                let is_system = path == SEELEN_COMMON.system_icon_pack_path();
                if is_system {
                    // Check before awaiting — never hold MutexGuard across an await point
                    let needs_load = self.system_icon_pack.lock().is_none();
                    if needs_load {
                        let mut icon_pack = IconPack::load(path).await?;
                        icon_pack.metadata.internal.bundled = true;
                        // we only read the system icon pack once, after that it is entirely runtime managed
                        *self.system_icon_pack.lock() = Some(icon_pack);
                    }
                    return Ok(None);
                }

                let icon_pack = IconPack::load(path).await?;
                let id = (*icon_pack.id).clone();
                self.icon_packs
                    .upsert(icon_pack.id.clone(), Arc::new(icon_pack));
                id
            }
            ResourceKind::SoundPack => {
                // feature not implemented
                return Ok(None);
            }
        };
        Ok(Some(id))
    }

    pub fn unload(&self, kind: &ResourceKind, path: &Path) {
        match kind {
            ResourceKind::Theme => {
                self.themes.retain(|_, v| v.metadata.internal.path != path);
            }
            ResourceKind::Widget => {
                self.plugins.retain(|_, v| v.metadata.internal.path != path);
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
                .retain(|_, v| self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::Plugin => self
                .plugins
                .retain(|_, v| self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::Widget => self
                .widgets
                .retain(|_, v| self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::IconPack => self
                .icon_packs
                .retain(|_, v| self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::Wallpaper => self
                .wallpapers
                .retain(|_, v| self.manual.contains(&v.metadata.internal.path)),
            ResourceKind::SoundPack => {
                // feature not implemented
            }
        }
    }

    /// returns a flat list of paths to be loaded for this kind
    async fn get_entries_for_type(kind: &ResourceKind) -> Result<Vec<PathBuf>> {
        let dirs: Vec<PathBuf> = match kind {
            ResourceKind::Theme => vec![
                SEELEN_COMMON.bundled_themes_path().to_path_buf(),
                SEELEN_COMMON.user_themes_path().to_path_buf(),
            ],
            ResourceKind::Widget => vec![
                SEELEN_COMMON.bundled_widgets_path().to_path_buf(),
                SEELEN_COMMON.user_widgets_path().to_path_buf(),
            ],
            ResourceKind::Plugin => vec![
                SEELEN_COMMON.bundled_plugins_path().to_path_buf(),
                SEELEN_COMMON.user_plugins_path().to_path_buf(),
            ],
            ResourceKind::Wallpaper => vec![SEELEN_COMMON.user_wallpapers_path().to_path_buf()],
            ResourceKind::IconPack => vec![SEELEN_COMMON.user_icons_path().to_path_buf()],
            ResourceKind::SoundPack => vec![SEELEN_COMMON.user_sounds_path().to_path_buf()],
        };

        async fn read_dir_entries(dir: PathBuf) -> Result<Vec<PathBuf>> {
            let mut rd = tokio::fs::read_dir(&dir).await?;
            let mut entries = Vec::new();
            while let Some(entry) = rd.next_entry().await? {
                entries.push(entry.path());
            }
            Ok(entries)
        }

        // read all dirs concurrently (kinds with bundled + user have 2 independent dirs)
        let results = futures::future::join_all(dirs.into_iter().map(read_dir_entries)).await;
        let mut paths = Vec::new();
        for result in results {
            paths.extend(result?);
        }
        Ok(paths)
    }

    pub async fn load_all_of_type(&self, kind: ResourceKind) -> Result<()> {
        log::trace!("Loading {kind:?}s");

        let paths = Self::get_entries_for_type(&kind).await?;
        self.unload_all(&kind);

        // spawn each path as an independent task for true multi-thread parallelism
        // (join_all drives all futures on the same thread; spawn distributes across the pool)
        let handles = paths
            .into_iter()
            .map(|path| tokio::spawn(async move { RESOURCES.load(&kind, &path).await }));
        for result in futures::future::join_all(handles).await {
            match result {
                Ok(Err(e)) => log::error!("Failed to load {kind:?}, error: {e}"),
                Err(e) => log::error!("Task panicked while loading {kind:?}: {e}"),
                Ok(Ok(_)) => {}
            }
        }

        if kind == ResourceKind::IconPack {
            // try load system icon pack
            let _ = self
                .load(&kind, SEELEN_COMMON.system_icon_pack_path())
                .await;
            // creates the system icon pack if not loaded
            self.ensure_system_icon_pack()?;
        }
        Ok(())
    }

    /// Activates a loaded resource the same way the "Enable" button does after installing
    /// a resource from the marketplace (via `seelen-ui.uri:` or a dropped `.slu` file).
    pub fn enable_resource(&self, kind: ResourceKind, id: ResourceId) {
        let plugin_events: Vec<PluginId> = match &kind {
            ResourceKind::Plugin => vec![PluginId::from(id.clone())],
            ResourceKind::Widget => {
                let widget_id = WidgetId::from(id.clone());
                self.widgets
                    .read(&widget_id, |_, w| {
                        w.plugins.iter().map(|p| p.id.clone()).collect()
                    })
                    .unwrap_or_default()
            }
            _ => vec![],
        };

        std::thread::spawn(move || {
            FULL_STATE.rcu(move |state| {
                let mut state = state.cloned();
                match kind {
                    ResourceKind::Theme => {
                        let theme_id = ThemeId::from(id.clone());
                        let has_shared_styles = RESOURCES
                            .themes
                            .read(&theme_id, |_, t| {
                                t.shared_styles.as_ref().is_some_and(|s| !s.is_empty())
                            })
                            .unwrap_or(false);
                        if has_shared_styles {
                            state.settings.active_themes.clear();
                            state.settings.active_themes.push("@default/theme".into());
                        }
                        state.settings.active_themes.push(theme_id);
                    }
                    ResourceKind::IconPack => {
                        state
                            .settings
                            .active_icon_packs
                            .push(IconPackId::from(id.clone()));
                    }
                    ResourceKind::Widget => {
                        state
                            .settings
                            .set_widget_enabled(&WidgetId::from(id.clone()), true);
                    }
                    ResourceKind::Wallpaper => {
                        let collection = WallpaperCollection {
                            id: Uuid::new_v4(),
                            name: "-".to_owned(),
                            wallpapers: vec![WallpaperId::from(id.clone())],
                            hidden: true,
                        };
                        state.settings.by_widget.wall.default_collection = Some(collection.id);
                        state.settings.wallpaper_collections.push(collection);
                    }
                    _ => {}
                }
                state
            });

            for plugin_id in plugin_events {
                emit_to_webviews(SeelenEvent::PluginEnabled, plugin_id);
            }

            FULL_STATE.load().write_settings().log_error();
        });
    }
}

unsafe impl Send for ResourceManager {}
unsafe impl Sync for ResourceManager {}
