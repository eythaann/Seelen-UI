use windows::Win32::Foundation::HWND;

use crate::{state::domain::AppConfig, windows_api::window::Window};

use super::FullState;

impl FullState {
    pub fn get_app_config_by_window(&self, hwnd: HWND) -> Option<&AppConfig> {
        let window = Window::from(hwnd);
        if let Ok(path) = window.process().program_path() {
            let title = window.title();
            let class = window.class();
            let exe = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let path = path.to_string_lossy().to_string();

            for app in self.settings_by_app.iter() {
                if app.identifier.validate(&title, &class, &exe, &path) {
                    return Option::from(app);
                }
            }
        }
        None
    }
}
