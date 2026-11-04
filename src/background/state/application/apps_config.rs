use seelen_core::state::{AppConfig, AppsConfigurationList};

use crate::{
    error::Result,
    platform::window::Window,
    state::application::{BUNDLED_SETTINGS_BY_APP, FULL_STATE},
    utils::constants::SEELEN_COMMON,
};

use super::AppSettings;

impl Window {
    /// Category assigned to the app of this window by its app config.
    pub fn slu_category(&self) -> Option<String> {
        self.get_app_config()
            .ok()?
            .and_then(|config| config.category.clone())
    }

    pub fn get_app_config(&self) -> Result<Option<AppConfig>> {
        let path = self.exe_path()?;

        let exe = path.file_name().ok_or("Invalid path")?.to_string_lossy();
        let path = path.to_string_lossy();
        let title = self.title();
        let class = self.class();

        if let Some(app) = FULL_STATE
            .load()
            .settings
            .by_app
            .search(&title, &class, &exe, &path)
        {
            return Ok(Some(app.clone()));
        }

        Ok(BUNDLED_SETTINGS_BY_APP
            .search(&title, &class, &exe, &path)
            .cloned())
    }
}

impl AppSettings {
    pub(super) fn load_bundled_settings_by_app() -> Result<AppsConfigurationList> {
        let apps_templates_path = SEELEN_COMMON.bundled_app_configs_path();
        let mut configs = AppsConfigurationList::default();

        for entry in apps_templates_path.read_dir()?.flatten() {
            let file = std::fs::File::open(entry.path())?;
            let mut apps: Vec<AppConfig> = serde_yaml::from_reader(&file)?;
            for app in apps.iter_mut() {
                app.is_bundled = true;
            }
            configs.extend(apps);
        }

        configs.prepare();
        Ok(configs)
    }
}
