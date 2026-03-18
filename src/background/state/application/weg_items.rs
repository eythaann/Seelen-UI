use seelen_core::{
    resource::WidgetId,
    state::{WegItem, WegItemData, WegItems},
};

use crate::{
    error::Result,
    modules::apps::application::msix::MsixAppsManager,
    utils::{atomic_write_file, constants::SEELEN_COMMON},
};

use super::FullState;

impl FullState {
    pub fn initial_weg_items() -> WegItems {
        WegItems {
            is_reorder_disabled: false,
            left: vec![
                WegItem::StartMenu {
                    id: uuid::Uuid::new_v4(),
                },
                WegItem::ShowDesktop {
                    id: uuid::Uuid::new_v4(),
                },
            ],
            center: vec![WegItem::AppOrFile(WegItemData {
                id: uuid::Uuid::new_v4(),
                umid: None,
                path: "C:\\Windows\\explorer.exe".into(),
                display_name: t!("file_explorer").to_string(),
                pinned: true,
                prevent_pinning: false,
                relaunch: None,
            })],
            right: vec![
                WegItem::TrashBin {
                    id: uuid::Uuid::new_v4(),
                },
                WegItem::Media {
                    id: uuid::Uuid::new_v4(),
                },
            ],
        }
    }

    fn update_weg_items_paths(items: &mut [WegItem]) {
        for item in items {
            if let WegItem::AppOrFile(data) = item {
                if let Some(umid) = &data.umid {
                    if let Ok(Some(app_path)) = MsixAppsManager::instance().get_app_path(umid) {
                        data.path = app_path;
                    }
                }
            }
        }
    }

    fn _read_weg_items(&mut self) -> Result<()> {
        let path = SEELEN_COMMON
            .widget_data_dir(&WidgetId::known_weg())
            .join("state.yml");

        if path.exists() {
            self.weg_items = serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
            self.weg_items.sanitize();
            Self::update_weg_items_paths(&mut self.weg_items.left);
            Self::update_weg_items_paths(&mut self.weg_items.center);
            Self::update_weg_items_paths(&mut self.weg_items.right);
        } else {
            self.weg_items = Self::initial_weg_items();
            self.weg_items.sanitize();
            self.write_weg_items(&self.weg_items)?;
        }
        Ok(())
    }

    pub(super) fn read_weg_items(&mut self) {
        if let Err(err) = self._read_weg_items() {
            log::error!("Failed to read weg items: {err}");
            Self::show_corrupted_state_to_user(
                &SEELEN_COMMON.widget_data_dir(&WidgetId::known_weg()),
            );
        }
    }

    pub fn write_weg_items(&self, items: &WegItems) -> Result<()> {
        let path = SEELEN_COMMON
            .widget_data_dir(&WidgetId::known_weg())
            .join("state.yml");
        atomic_write_file(&path, serde_yaml::to_string(items)?.as_bytes())
    }
}
