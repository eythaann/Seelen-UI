use windows::Win32::Foundation::HWND;

use crate::{state::domain::AppConfig, windows_api::WindowsApi};

use super::FullState;

impl FullState {
    pub fn get_app_config_by_window(&self, hwnd: HWND) -> Option<&AppConfig> {
        // Can no cache apps that changes titles
        /* match self.cache.entry(hwnd.0) {
            Entry::Occupied(entry) => entry.get().and_then(|index| self.apps.get(index)),
            Entry::Vacant(entry) => {
                for (i, app) in self.apps.iter().enumerate() {
                    if app.match_window(hwnd) {
                        entry.insert(Some(i));
                        return Option::from(app);
                    }
                }
                entry.insert(None);
                None
            }
        } */

        if let (title, Ok(path), Ok(exe), Ok(class)) = (
            WindowsApi::get_window_text(hwnd),
            WindowsApi::exe_path(hwnd),
            WindowsApi::exe(hwnd),
            WindowsApi::get_class(hwnd),
        ) {
            for app in self.settings_by_app.iter() {
                if app.identifier.validate(&title, &class, &exe, &path) {
                    return Option::from(app);
                }
            }
        }

        None
    }
}
