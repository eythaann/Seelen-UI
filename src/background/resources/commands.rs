use seelen_core::{
    resource::{ResourceId, ResourceKind, SluResource},
    state::{IconPack, Plugin, Theme, Wallpaper, Widget},
};

use crate::{error::Result, log_error, resources::RESOURCES};
use std::sync::Arc;

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
pub fn state_get_themes() -> Vec<Arc<Theme>> {
    let mut themes = Vec::new();
    RESOURCES.themes.scan(|_, v| {
        themes.push(v.clone());
    });
    themes
}

#[tauri::command(async)]
pub fn state_get_plugins() -> Vec<Arc<Plugin>> {
    let mut plugins = Vec::new();
    RESOURCES.plugins.scan(|_, v| {
        plugins.push(v.clone());
    });
    plugins
}

#[tauri::command(async)]
pub fn state_get_widgets() -> Vec<Arc<Widget>> {
    let mut widgets = Vec::new();
    RESOURCES.widgets.scan(|_, v| {
        widgets.push(v.clone());
    });
    widgets
}

#[tauri::command(async)]
pub fn state_get_wallpapers() -> Vec<Arc<Wallpaper>> {
    let mut wallpapers = Vec::new();
    RESOURCES.wallpapers.scan(|_, v| {
        wallpapers.push(v.clone());
    });
    wallpapers
}

#[tauri::command(async)]
pub fn state_get_icon_packs() -> Vec<Arc<IconPack>> {
    let mut icon_packs = Vec::new();

    // Add system icon pack if it exists
    if let Some(system_pack) = RESOURCES.system_icon_pack.lock().as_ref() {
        icon_packs.push(Arc::new(system_pack.clone()));
    }

    // Add user icon packs
    RESOURCES.icon_packs.scan(|_, v| {
        icon_packs.push(v.clone());
    });

    icon_packs
}

#[tauri::command(async)]
pub fn state_delete_cached_icons() -> Result<()> {
    if let Some(pack) = RESOURCES.system_icon_pack.lock().take() {
        pack.delete()?;
    }
    RESOURCES.ensure_system_icon_pack()?;
    RESOURCES.emit_icon_packs()?;
    Ok(())
}
