mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, state::NativeWallpaperSnapshot};
use tauri::Manager;

use crate::{
    app::{emit_to_webviews, get_app_handle},
    error::ResultLogExt,
};

use self::{application::ShellWallpaperManager, domain::ShellWallpaperEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static ShellWallpaperManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        ShellWallpaperManager::subscribe(|event| match event {
            ShellWallpaperEvent::Changed { snapshot } => {
                emit_to_webviews(
                    SeelenEvent::NativeShellWallpaperChanged,
                    expose_snapshot(snapshot),
                );
            }
        });
    });
    ShellWallpaperManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_native_shell_wallpaper_snapshot() -> NativeWallpaperSnapshot {
        expose_snapshot(get_manager().snapshot())
    }
}

/// The wallpaper can live anywhere and may have no extension (`TranscodedWallpaper`), so the
/// static asset scope can't cover it. Exactly the current file is allowed instead of widening the
/// scope to whole folders or extensions.
fn expose_snapshot(snapshot: NativeWallpaperSnapshot) -> NativeWallpaperSnapshot {
    if !snapshot.path.as_os_str().is_empty() {
        get_app_handle()
            .asset_protocol_scope()
            .allow_file(&snapshot.path)
            .log_error();
    }
    snapshot
}
