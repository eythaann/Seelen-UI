use std::sync::Arc;

use seelen_core::{
    handlers::SeelenEvent,
    resource::ResourceKind,
    state::{settings::shortcuts::resolve_shortcuts, Theme, Widget},
};
use slu_ipc::messages::SvcAction;

use crate::{
    app::emit_to_webviews, cli::ServicePipe, error::Result, session::application::SessionManager,
    state::application::FULL_STATE, widgets::manager::WIDGET_MANAGER,
};

use super::ResourceManager;

// Reusable getters for resources.
impl ResourceManager {
    pub fn widgets(&self) -> Vec<Arc<Widget>> {
        let show_premiums = SessionManager::instance().lock().has_premium_access();
        let mut widgets = Vec::new();
        self.widgets.scan(|_, v| {
            if v.metadata.premium && !show_premiums {
                return;
            }
            widgets.push(v.clone());
        });
        widgets
    }

    pub fn themes(&self) -> Vec<Arc<Theme>> {
        let show_premiums = SessionManager::instance().lock().has_premium_access();
        let mut themes = Vec::new();
        self.themes.scan(|_, v| {
            if v.metadata.premium && !show_premiums {
                return;
            }
            themes.push(v.clone());
        });
        themes
    }

    pub fn plugins(&self) -> Vec<Arc<seelen_core::state::Plugin>> {
        let show_premiums = SessionManager::instance().lock().has_premium_access();
        let mut plugins = Vec::new();
        self.plugins.scan(|_, v| {
            if v.metadata.premium && !show_premiums {
                return;
            }
            plugins.push(v.clone());
        });
        plugins
    }

    pub fn icon_packs(&self) -> Vec<Arc<seelen_core::state::IconPack>> {
        let show_premiums = SessionManager::instance().lock().has_premium_access();
        let mut icon_packs = Vec::new();
        // Add system icon pack if it exists
        if let Some(system_pack) = self.system_icon_pack.lock().as_ref() {
            icon_packs.push(std::sync::Arc::new(system_pack.clone()));
        }
        // Add user icon packs
        self.icon_packs.scan(|_, v| {
            if v.metadata.premium && !show_premiums {
                return;
            }
            icon_packs.push(v.clone());
        });
        icon_packs
    }

    pub fn wallpapers(&self) -> Vec<Arc<seelen_core::state::Wallpaper>> {
        let show_premiums = SessionManager::instance().lock().has_premium_access();
        let mut wallpapers = Vec::new();
        self.wallpapers.scan(|_, v| {
            if v.metadata.premium && !show_premiums {
                return;
            }
            wallpapers.push(v.clone());
        });
        wallpapers
    }
}

impl ResourceManager {
    pub fn emit_widgets(&self) -> Result<()> {
        let widgets = self.widgets();
        emit_to_webviews(SeelenEvent::StateWidgetsChanged, widgets.clone());

        WIDGET_MANAGER.reconcile()?;

        let state = FULL_STATE.load();
        let widget_refs: Vec<_> = widgets.iter().map(|w| w.as_ref()).collect();
        let resolved = resolve_shortcuts(&state.settings, &widget_refs);
        ServicePipe::request(SvcAction::SetShortcuts(resolved))?;
        Ok(())
    }

    pub fn emit_themes(&self) {
        emit_to_webviews(SeelenEvent::StateThemesChanged, self.themes())
    }

    pub fn emit_plugins(&self) {
        emit_to_webviews(SeelenEvent::StatePluginsChanged, self.plugins())
    }

    pub fn emit_icon_packs(&self) {
        emit_to_webviews(SeelenEvent::StateIconPacksChanged, self.icon_packs())
    }

    pub fn emit_wallpapers(&self) {
        emit_to_webviews(SeelenEvent::StateWallpapersChanged, self.wallpapers())
    }

    pub fn emit_kind_changed(&self, kind: &ResourceKind) -> Result<()> {
        match kind {
            ResourceKind::Theme => self.emit_themes(),
            ResourceKind::Widget => {
                self.emit_plugins();
                self.emit_widgets()?;
            }
            ResourceKind::Plugin => self.emit_plugins(),
            ResourceKind::IconPack => self.emit_icon_packs(),
            ResourceKind::Wallpaper => self.emit_wallpapers(),
            ResourceKind::SoundPack => {
                // feature not implemented
            }
        }
        Ok(())
    }

    /// Emits change events only for resource types that contain at least one
    /// premium item, since those are the only lists whose contents change when
    /// premium access is gained or lost.
    pub fn emit_on_session_changed(&self) -> Result<()> {
        let mut has_premium = false;

        self.themes.scan(|_, v| has_premium |= v.metadata.premium);
        if has_premium {
            self.emit_themes();
            has_premium = false;
        }

        self.plugins.scan(|_, v| has_premium |= v.metadata.premium);
        if has_premium {
            self.emit_plugins();
            has_premium = false;
        }

        self.widgets.scan(|_, v| has_premium |= v.metadata.premium);
        if has_premium {
            self.emit_widgets()?;
            has_premium = false;
        }

        self.icon_packs
            .scan(|_, v| has_premium |= v.metadata.premium);
        if has_premium {
            self.emit_icon_packs();
            has_premium = false;
        }

        self.wallpapers
            .scan(|_, v| has_premium |= v.metadata.premium);
        if has_premium {
            self.emit_wallpapers();
        }

        Ok(())
    }
}
