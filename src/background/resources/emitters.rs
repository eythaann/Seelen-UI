use seelen_core::{handlers::SeelenEvent, resource::ResourceKind};
use tauri::Emitter;

use crate::{app::get_app_handle, error::Result};

use super::ResourceManager;

impl ResourceManager {
    pub fn emit_widgets(&self) -> Result<()> {
        let mut widgets = Vec::new();
        self.widgets.scan(|_, v| {
            widgets.push(v.clone());
        });
        get_app_handle().emit(SeelenEvent::StateWidgetsChanged, widgets)?;
        Ok(())
    }

    pub fn emit_themes(&self) -> Result<()> {
        let mut themes = Vec::new();
        self.themes.scan(|_, v| {
            themes.push(v.clone());
        });
        get_app_handle().emit(SeelenEvent::StateThemesChanged, themes)?;
        Ok(())
    }

    pub fn emit_plugins(&self) -> Result<()> {
        let mut plugins = Vec::new();
        self.plugins.scan(|_, v| {
            plugins.push(v.clone());
        });
        get_app_handle().emit(SeelenEvent::StatePluginsChanged, plugins)?;
        Ok(())
    }

    pub fn emit_icon_packs(&self) -> Result<()> {
        let mut icon_packs = Vec::new();
        self.icon_packs.scan(|_, v| {
            icon_packs.push(v.clone());
        });
        get_app_handle().emit(SeelenEvent::StateIconPacksChanged, icon_packs)?;
        Ok(())
    }

    pub fn emit_wallpapers(&self) -> Result<()> {
        let mut wallpaper = Vec::new();
        self.wallpapers.scan(|_, v| {
            wallpaper.push(v.clone());
        });
        get_app_handle().emit(SeelenEvent::StateWallpapersChanged, wallpaper)?;
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
