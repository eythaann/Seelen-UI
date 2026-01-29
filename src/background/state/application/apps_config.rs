use windows::Win32::Foundation::HWND;

use crate::{
    error::Result, state::domain::AppConfig, utils::constants::SEELEN_COMMON,
    windows_api::window::Window,
};

use super::FullState;

impl FullState {
    pub fn get_app_config_by_window(&self, hwnd: HWND) -> Result<Option<&AppConfig>> {
        let window = Window::from(hwnd);

        let path = window.process().program_path()?;

        let exe = path.file_name().ok_or("Invalid path")?.to_string_lossy();
        let path = path.to_string_lossy();
        let title = window.title();
        let class = window.class();

        if let Some(app) = self.settings.by_app.search(&title, &class, &exe, &path) {
            return Ok(Some(app));
        }

        Ok(self.settings_by_app.search(&title, &class, &exe, &path))
    }

    fn _load_bundled_settings_by_app(&mut self) -> Result<()> {
        let apps_templates_path = SEELEN_COMMON.bundled_app_configs_path();

        self.settings_by_app.clear();

        for entry in apps_templates_path.read_dir()?.flatten() {
            let content = std::fs::read_to_string(entry.path())?;
            let mut apps: Vec<AppConfig> = serde_yaml::from_str(&content)?;
            for app in apps.iter_mut() {
                app.is_bundled = true;
            }
            self.settings_by_app.extend(apps);
        }

        self.settings_by_app.prepare();
        Ok(())
    }

    pub(super) fn load_bundled_settings_by_app(&mut self) {
        if let Err(e) = self._load_bundled_settings_by_app() {
            log::error!("Error loading settings by app: {e}");
        }
    }
}
