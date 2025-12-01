use windows::Win32::Foundation::HWND;

use crate::{error::Result, state::domain::AppConfig, windows_api::window::Window};

use super::FullState;

impl FullState {
    pub fn get_app_config_by_window(&self, hwnd: HWND) -> Result<Option<&AppConfig>> {
        let window = Window::from(hwnd);

        let path = window.process().program_path()?;

        let exe = path.file_name().ok_or("Invalid path")?.to_string_lossy();
        let path = path.to_string_lossy();
        let title = window.title();
        let class = window.class();

        Ok(self.settings_by_app.search(&title, &class, &exe, &path))
    }
}
