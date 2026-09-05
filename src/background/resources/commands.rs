use seelen_core::{
    resource::{ResourceId, ResourceKind, SluResource},
    state::{
        IconPack, Plugin, Theme, Wallpaper, Widget, settings::shortcuts::SystemShortcutDeclaration,
    },
};

use crate::{
    error::{Result, ResultLogExt},
    resources::RESOURCES,
    utils::icon_extractor::queue::IconExtractor,
};
use std::{path::PathBuf, sync::Arc};

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

async fn delete_path(path: PathBuf) {
    if path.is_dir() {
        tokio::fs::remove_dir_all(&path).await.log_error();
    } else {
        tokio::fs::remove_file(&path).await.log_error();
    }
}

#[tauri::command(async)]
pub async fn remove_resource(kind: ResourceKind, id: ResourceId) -> Result<()> {
    let mut to_delete = Vec::new();

    match kind {
        ResourceKind::Theme => {
            RESOURCES
                .themes
                .retain_async(|_, v| {
                    if *v.id == id && !v.metadata.internal.bundled {
                        to_delete.push(v.metadata.internal.path.clone());
                        return false;
                    }
                    true
                })
                .await;
        }
        ResourceKind::Plugin => {
            RESOURCES
                .plugins
                .retain_async(|_, v| {
                    if *v.id == id && !v.metadata.internal.bundled {
                        to_delete.push(v.metadata.internal.path.clone());
                        return false;
                    }
                    true
                })
                .await;
        }
        ResourceKind::Widget => {
            RESOURCES
                .widgets
                .retain_async(|_, v| {
                    if *v.id == id && !v.metadata.internal.bundled {
                        to_delete.push(v.metadata.internal.path.clone());
                        return false;
                    }
                    true
                })
                .await;
        }
        ResourceKind::IconPack => {
            RESOURCES
                .icon_packs
                .retain_async(|_, v| {
                    if *v.id == id && !v.metadata.internal.bundled {
                        to_delete.push(v.metadata.internal.path.clone());
                        return false;
                    }
                    true
                })
                .await;
        }
        ResourceKind::Wallpaper => {
            RESOURCES
                .wallpapers
                .retain_async(|_, v| {
                    if *v.id == id && !v.metadata.internal.bundled {
                        to_delete.push(v.metadata.internal.path.clone());
                        return false;
                    }
                    true
                })
                .await;
        }
        ResourceKind::SoundPack => {
            // feature not implemented
        }
    }

    for path in to_delete {
        delete_path(path).await;
    }

    RESOURCES.emit_kind_changed(&kind)?;
    Ok(())
}

#[tauri::command(async)]
pub async fn state_delete_cached_icons() -> Result<()> {
    // Take the pack before awaiting — never hold MutexGuard across an await point
    let pack = { RESOURCES.system_icon_pack.lock().take() };
    if let Some(pack) = pack {
        pack.delete().await?;
    }
    IconExtractor::instance().clear_failures();
    RESOURCES.ensure_system_icon_pack(true)?;
    RESOURCES.emit_icon_packs();
    Ok(())
}
