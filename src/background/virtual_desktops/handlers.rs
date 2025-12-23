use std::sync::Once;

use seelen_core::{
    handlers::SeelenEvent,
    resource::{SluResource, WallpaperId},
    state::{VirtualDesktops, Wallpaper},
    system_state::MonitorId,
};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    resources::RESOURCES,
    utils::date_based_hex_id,
    virtual_desktops::SluWorkspacesManager2,
};

fn get_vd_manager() -> &'static SluWorkspacesManager2 {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        SluWorkspacesManager2::subscribe(|_event| {
            let payload: VirtualDesktops = SluWorkspacesManager2::instance().into();
            get_app_handle()
                .emit(SeelenEvent::VirtualDesktopsChanged, payload)
                .log_error();
        });
    });

    SluWorkspacesManager2::instance()
}

#[tauri::command(async)]
pub fn get_virtual_desktops() -> VirtualDesktops {
    get_vd_manager().into()
}

#[tauri::command(async)]
pub fn switch_workspace(monitor_id: MonitorId, idx: usize) -> Result<()> {
    get_vd_manager().switch_to(&monitor_id, idx)
}

#[tauri::command(async)]
pub fn wallpaper_next() {
    super::wallpapers::WorkspaceWallpapersManager::next();
}

#[tauri::command(async)]
pub fn wallpaper_prev() {
    super::wallpapers::WorkspaceWallpapersManager::previous();
}

#[tauri::command(async)]
pub fn wallpaper_save_thumbnail(wallpaper_id: WallpaperId, thumbnail_bytes: Vec<u8>) -> Result<()> {
    let Some(wallpaper) = RESOURCES.wallpapers.get(&wallpaper_id) else {
        return Err("Invalid wallpaper id".into());
    };

    let thumbnail_filename = format!("thumbnail_{}.jpg", date_based_hex_id());
    let thumbnail_path = wallpaper.metadata.directory()?.join(&thumbnail_filename);
    std::fs::write(&thumbnail_path, &thumbnail_bytes)?;

    let mut wallpaper_mut = Wallpaper::clone(&wallpaper);
    wallpaper_mut.thumbnail_filename = Some(thumbnail_filename);
    wallpaper_mut.save()?;
    Ok(())
}
