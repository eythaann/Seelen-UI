use seelen_core::{handlers::SeelenEvent, resource::SluResource, state::Wallpaper};
use tauri::Emitter;

use crate::{
    app::get_app_handle,
    error_handler::Result,
    utils::{constants::SEELEN_COMMON, date_based_hex_id},
};

use super::FullState;

impl FullState {
    pub(super) fn emit_wallpapers(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateWallpapersChanged, &self.wallpapers)?;
        Ok(())
    }

    pub(super) fn load_wallpapers(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(SEELEN_COMMON.user_wallpapers_path())?;
        self.wallpapers.clear();
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_file() {
                let Some(extension) = path.extension() else {
                    continue;
                };

                let extension = extension.to_string_lossy().to_lowercase();
                if Wallpaper::SUPPORTED_IMAGES.contains(&extension.as_ref())
                    || Wallpaper::SUPPORTED_VIDEOS.contains(&extension.as_ref())
                {
                    match Wallpaper::create_from_file(
                        &path,
                        &SEELEN_COMMON
                            .user_wallpapers_path()
                            .join(date_based_hex_id()),
                        false,
                    ) {
                        Ok(wallpaper) => {
                            self.wallpapers.push(wallpaper);
                        }
                        Err(e) => {
                            log::error!("Failed to load wallpaper: {e}");
                        }
                    }
                }
                continue;
            }

            match Wallpaper::load(&path) {
                Ok(wallpaper) => {
                    self.wallpapers.push(wallpaper);
                }
                Err(e) => {
                    log::error!("Failed to load wallpaper: {e}");
                }
            }
        }
        Ok(())
    }
}
