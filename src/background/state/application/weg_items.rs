use std::{
    fs::{create_dir_all, OpenOptions},
    io::Write,
};

use seelen_core::{
    handlers::SeelenEvent,
    resource::WidgetId,
    state::{WegItem, WegItems},
};

use crate::{
    app::emit_to_webviews, error::Result, modules::apps::application::msix::MsixAppsManager,
    trace_lock, utils::constants::SEELEN_COMMON, widgets::weg::weg_items_impl::SEELEN_WEG_STATE,
};

use super::FullState;

impl FullState {
    fn update_weg_items_paths(items: &mut [WegItem]) {
        for item in items {
            match item {
                WegItem::Pinned(data) | WegItem::Temporal(data) => {
                    if let Some(umid) = &data.umid {
                        if let Ok(Some(app_path)) = MsixAppsManager::instance().get_app_path(umid) {
                            data.path = app_path;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn emit_weg_items(&self) -> Result<()> {
        emit_to_webviews(SeelenEvent::StateWegItemsChanged, &self.weg_items);
        trace_lock!(SEELEN_WEG_STATE).on_stored_changed(self.weg_items.clone())?;
        Ok(())
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
        let dir = SEELEN_COMMON.widget_data_dir(&WidgetId::known_weg());
        create_dir_all(&dir)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dir.join("state.yml"))?;
        file.lock()?;
        file.write_all(serde_yaml::to_string(items)?.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}
