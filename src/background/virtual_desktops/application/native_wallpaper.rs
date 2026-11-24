//! Mirrors the wallpaper of the primary monitor's active workspace on the native desktop wallpaper,
//! so system surfaces that show it (UAC prompt, session transitions, etc.) match Seelen UI.
//! The service applies it and restores the user's own wallpaper when the app closes.

use std::path::PathBuf;
use std::sync::LazyLock;

use seelen_core::resource::{SluResource, WidgetId};
use seelen_core::state::{Wallpaper, WallpaperKind};
use slu_ipc::messages::SvcAction;

use crate::cli::ServicePipe;
use crate::error::{Result, ResultLogExt};
use crate::platform::monitor::Monitor;
use crate::resources::RESOURCES;
use crate::state::FULL_STATE;
use crate::utils::lock_free::TracedMutex;

use super::VdManager;

/// Image formats the native wallpaper engine can display without extra codecs.
const NATIVE_IMAGE_EXTENSIONS: [&str; 9] = [
    "jpg", "jpeg", "jfif", "png", "bmp", "dib", "gif", "tif", "tiff",
];

/// Last request sent to the service, to not send the same one on every workspace event.
static LAST_REQUEST: LazyLock<TracedMutex<Option<NativeWallpaperRequest>>> =
    LazyLock::new(|| TracedMutex::new(None));

#[derive(Debug, Clone, PartialEq, Eq)]
enum NativeWallpaperRequest {
    /// `None` = black.
    Set(Option<PathBuf>),
    Restore,
}

/// Recomputes the native wallpaper and sends it to the service if it changed.
pub fn sync() {
    let request = if is_wall_enabled_on_primary() {
        NativeWallpaperRequest::Set(primary_static_wallpaper())
    } else {
        NativeWallpaperRequest::Restore
    };

    {
        let mut last = LAST_REQUEST.lock();
        if last.as_ref() == Some(&request) {
            return;
        }
        *last = Some(request.clone());
    }

    log::trace!("Native wallpaper: {request:?}");
    let action = match request {
        NativeWallpaperRequest::Set(path) => SvcAction::SetNativeWallpaper { path },
        NativeWallpaperRequest::Restore => SvcAction::RestoreNativeWallpaper,
    };
    ServicePipe::request(action).log_error();
}

fn is_wall_enabled_on_primary() -> bool {
    let state = FULL_STATE.load();
    let wall = WidgetId::known_wall();
    match Monitor::primary().stable_id() {
        Ok(monitor_id) => state.is_widget_enable_on_monitor(&wall, &monitor_id),
        Err(_) => state.is_widget_enabled(&wall),
    }
}

/// Static image of the wallpaper shown on the primary monitor, `None` if there is none.
fn primary_static_wallpaper() -> Option<PathBuf> {
    let monitor_id = Monitor::primary().stable_id().ok()?;
    let wallpaper_id = VdManager::instance()
        .monitors
        .get(&monitor_id, |monitor| {
            monitor.active_workspace().wallpaper.clone()
        })??;
    let wallpaper = RESOURCES.wallpapers.get_sync(&wallpaper_id)?;
    static_image_of(&wallpaper)
        .inspect_err(|err| log::error!("Failed to resolve the native wallpaper: {err:?}"))
        .ok()
        .flatten()
}

/// The image itself for image wallpapers, the thumbnail for the others (videos, layered, etc.)
/// or when the image format is not supported natively.
fn static_image_of(wallpaper: &Wallpaper) -> Result<Option<PathBuf>> {
    let image = match wallpaper.r#type {
        WallpaperKind::Image => wallpaper.filename.as_ref(),
        _ => None,
    };
    let directory = wallpaper.metadata().directory()?;
    Ok(image
        .into_iter()
        .chain(wallpaper.thumbnail_filename.as_ref())
        .map(|filename| directory.join(filename))
        .find(|path| is_native_image(path) && path.is_file()))
}

fn is_native_image(path: &std::path::Path) -> bool {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_lowercase())
        .is_some_and(|ext| NATIVE_IMAGE_EXTENSIONS.contains(&ext.as_str()))
}
