use seelen_core::handlers::SeelenEvent;
use tauri::Emitter;

use crate::{error_handler::Result, windows_api::window::Window, winevent::WinEvent};

use super::SeelenWall;

impl SeelenWall {
    pub fn process_win_event(&mut self, event: WinEvent, origin: &Window) -> Result<()> {
        match event {
            WinEvent::SyntheticFullscreenStart => {
                if !origin.is_seelen_overlay() {
                    // todo handle this by monitor
                    self.window
                        .emit_to(self.window.label(), SeelenEvent::WallStop, true)?;
                }
            }
            WinEvent::SyntheticFullscreenEnd => {
                // todo handle this by monitor
                self.window
                    .emit_to(self.window.label(), SeelenEvent::WallStop, false)?;
            }
            _ => {}
        }
        Ok(())
    }
}
