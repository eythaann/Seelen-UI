use std::{fs::OpenOptions, io::Write};

use seelen_core::{
    handlers::SeelenEvent,
    state::{GenericToolbarItem, Placeholder, TextToolbarItem, ToolbarItem, ToolbarItem2},
};
use tauri::Emitter;

use crate::{error_handler::Result, seelen::get_app_handle, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    fn initial_toolbar_items() -> Placeholder {
        Placeholder {
            left: vec![
                ToolbarItem2::Plugin("@default/user-folder".into()),
                ToolbarItem2::Inline(ToolbarItem::Text(TextToolbarItem {
                    template: "\"|\"".into(),
                    ..Default::default()
                })),
                ToolbarItem2::Plugin("@default/focused-app".into()),
                ToolbarItem2::Inline(ToolbarItem::Generic(GenericToolbarItem {
                    template: "window.title ? \"-\" : \"\"".into(),
                    ..Default::default()
                })),
                ToolbarItem2::Plugin("@default/focused-app-title".into()),
            ],
            center: vec![ToolbarItem2::Plugin("@default/date".into())],
            right: vec![
                ToolbarItem2::Plugin("@default/system-tray".into()),
                ToolbarItem2::Plugin("@default/bluetooth".into()),
                ToolbarItem2::Plugin("@default/network".into()),
                ToolbarItem2::Plugin("@default/media".into()),
                ToolbarItem2::Plugin("@default/power".into()),
                ToolbarItem2::Plugin("@default/notifications".into()),
                ToolbarItem2::Plugin("@default/quick-settings".into()),
            ],
            ..Default::default()
        }
    }

    pub fn emit_toolbar_items(&self) -> Result<()> {
        get_app_handle().emit(SeelenEvent::StateToolbarItemsChanged, &self.toolbar_items)?;
        Ok(())
    }

    pub(super) fn read_toolbar_items(&mut self) -> Result<()> {
        if SEELEN_COMMON.toolbar_items_path().exists() {
            self.toolbar_items = serde_yaml::from_str(&std::fs::read_to_string(
                SEELEN_COMMON.toolbar_items_path(),
            )?)?;
            self.toolbar_items.sanitize();
        } else {
            self.toolbar_items = Self::initial_toolbar_items();
            self.toolbar_items.sanitize();
            self.write_toolbar_items(&self.toolbar_items)?;
        }
        Ok(())
    }

    pub fn write_toolbar_items(&self, items: &Placeholder) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(SEELEN_COMMON.toolbar_items_path())?;
        file.write_all(serde_yaml::to_string(items)?.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}
