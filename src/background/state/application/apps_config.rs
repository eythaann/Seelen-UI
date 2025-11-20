use windows::Win32::Foundation::HWND;

use crate::{state::domain::AppConfig, windows_api::window::Window};

use super::FullState;

impl FullState {
    pub fn get_app_config_by_window(&self, hwnd: HWND) -> Option<&AppConfig> {
        let window = Window::from(hwnd);

        let path = window.process().program_path().ok()?;

        let exe = path.file_name()?.to_string_lossy();
        let path = path.to_string_lossy();
        let title = window.title();
        let class = window.class();

        self.settings_by_app.search(&title, &class, &exe, &path)
    }
}
