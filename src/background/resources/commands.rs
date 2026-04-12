use seelen_core::{
    resource::{ResourceId, ResourceKind, SluResource},
    state::{
        settings::shortcuts::SystemShortcutDeclaration, IconPack, Plugin, Theme, Wallpaper, Widget,
    },
};

use crate::{
    error::Result, log_error, resources::RESOURCES, utils::icon_extractor::queue::IconExtractor,
};
use std::sync::Arc;

#[tauri::command(async)]
pub fn state_get_widgets() -> Vec<Arc<Widget>> {
    RESOURCES.widgets()
}

#[tauri::command(async)]
pub fn state_get_system_shortcuts() -> Vec<SystemShortcutDeclaration> {
    seelen_core::state::settings::shortcuts::system_shortcut_declarations()
}

#[tauri::command(async)]
pub fn state_get_themes() -> Vec<Arc<Theme>> {
    RESOURCES.themes()
}

#[tauri::command(async)]
pub fn state_get_plugins() -> Vec<Arc<Plugin>> {
    RESOURCES.plugins()
}

#[tauri::command(async)]
pub fn state_get_wallpapers() -> Vec<Arc<Wallpaper>> {
    RESOURCES.wallpapers()
}

#[tauri::command(async)]
pub fn state_get_icon_packs() -> Vec<Arc<IconPack>> {
    RESOURCES.icon_packs()
}

#[tauri::command(async)]
pub fn remove_resource(kind: ResourceKind, id: ResourceId) -> Result<()> {
    match kind {
        ResourceKind::Theme => {
            RESOURCES.themes.retain(|_, v| {
                if *v.id == id && !v.metadata.internal.bundled {
                    log_error!(v.delete());
                    return false;
                }
                true
            });
        }
        ResourceKind::Plugin => {
            RESOURCES.plugins.retain(|_, v| {
                if *v.id == id && !v.metadata.internal.bundled {
                    log_error!(v.delete());
                    return false;
                }
                true
            });
        }
        ResourceKind::Widget => {
            RESOURCES.widgets.retain(|_, v| {
                if *v.id == id && !v.metadata.internal.bundled {
                    log_error!(v.delete());
                    return false;
                }
                true
            });
        }
        ResourceKind::IconPack => {
            RESOURCES.icon_packs.retain(|_, v| {
                if *v.id == id && !v.metadata.internal.bundled {
                    log_error!(v.delete());
                    return false;
                }
                true
            });
        }
        ResourceKind::Wallpaper => {
            RESOURCES.wallpapers.retain(|_, v| {
                if *v.id == id && !v.metadata.internal.bundled {
                    log_error!(v.delete());
                    return false;
                }
                true
            });
        }
        ResourceKind::SoundPack => {
            // feature not implemented
        }
    }
    RESOURCES.emit_kind_changed(&kind)?;
    Ok(())
}

#[tauri::command(async)]
pub fn state_delete_cached_icons() -> Result<()> {
    if let Some(pack) = RESOURCES.system_icon_pack.lock().take() {
        pack.delete()?;
    }
    IconExtractor::instance().clear_failures();
    RESOURCES.ensure_system_icon_pack()?;
    RESOURCES.emit_icon_packs();
    Ok(())
}
