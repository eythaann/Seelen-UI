use crate::{error::Result, windows_api::window::event::WinEvent, windows_api::window::Window};

use super::SeelenWall;

impl SeelenWall {
    pub fn process_win_event(&mut self, _event: WinEvent, _origin: &Window) -> Result<()> {
        Ok(())
    }
}
