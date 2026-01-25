use seelen_core::{handlers::SeelenEvent, resource::ResourceKind};

use crate::{app::emit_to_webviews, error::Result, widgets::manager::WIDGET_MANAGER};

use super::ResourceManager;

impl ResourceManager {
    pub fn emit_widgets(&self) -> Result<()> {
        let mut widgets = Vec::new();
        self.widgets.scan(|_, v| {
            widgets.push(v.clone());
        });
        emit_to_webviews(SeelenEvent::StateWidgetsChanged, widgets);
        WIDGET_MANAGER.refresh()?;
        Ok(())
    }

    pub fn emit_themes(&self) -> Result<()> {
        let mut themes = Vec::new();
        self.themes.scan(|_, v| {
            themes.push(v.clone());
        });
        emit_to_webviews(SeelenEvent::StateThemesChanged, themes);
        Ok(())
    }

    pub fn emit_plugins(&self) -> Result<()> {
        let mut plugins = Vec::new();
        self.plugins.scan(|_, v| {
            plugins.push(v.clone());
        });
        emit_to_webviews(SeelenEvent::StatePluginsChanged, plugins);
        Ok(())
    }

    pub fn emit_icon_packs(&self) -> Result<()> {
        let mut icon_packs = Vec::new();

        // Add system icon pack if it exists
        if let Some(system_pack) = self.system_icon_pack.lock().as_ref() {
            icon_packs.push(std::sync::Arc::new(system_pack.clone()));
        }

        // Add user icon packs
        self.icon_packs.scan(|_, v| {
            icon_packs.push(v.clone());
        });

        emit_to_webviews(SeelenEvent::StateIconPacksChanged, icon_packs);
        Ok(())
    }

    pub fn emit_wallpapers(&self) -> Result<()> {
        let mut wallpaper = Vec::new();
        self.wallpapers.scan(|_, v| {
            wallpaper.push(v.clone());
        });
        emit_to_webviews(SeelenEvent::StateWallpapersChanged, wallpaper);
        Ok(())
    }

    pub fn emit_kind_changed(&self, kind: &ResourceKind) -> Result<()> {
        match kind {
            ResourceKind::Theme => self.emit_themes()?,
            ResourceKind::Widget => self.emit_widgets()?,
            ResourceKind::Plugin => self.emit_plugins()?,
            ResourceKind::IconPack => self.emit_icon_packs()?,
            ResourceKind::Wallpaper => self.emit_wallpapers()?,
            ResourceKind::SoundPack => {
                // feature not implemented
            }
        }
        Ok(())
    }
}
