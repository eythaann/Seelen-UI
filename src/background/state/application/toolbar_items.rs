use std::{fs::OpenOptions, io::Write};

use seelen_core::{
    handlers::SeelenEvent,
    state::{GenericToolbarItem, Placeholder, TextToolbarItem, ToolbarItem, ToolbarItem2},
};

use crate::{app::emit_to_webviews, error::Result, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    pub fn initial_toolbar_items() -> Placeholder {
        Placeholder {
            left: vec![
                ToolbarItem2::Plugin("@seelen/tb-user-menu".into()),
                ToolbarItem2::Inline(Box::new(ToolbarItem::Text(TextToolbarItem {
                    template: "return \"|\"".into(),
                    ..Default::default()
                }))),
                ToolbarItem2::Plugin("@default/focused-app".into()),
                ToolbarItem2::Inline(Box::new(ToolbarItem::Generic(GenericToolbarItem {
                    template: "return window.title ? \"-\" : \"\"".into(),
                    ..Default::default()
                }))),
                ToolbarItem2::Plugin("@default/focused-app-title".into()),
            ],
            center: vec![ToolbarItem2::Plugin("@seelen/tb-calendar-popup".into())],
            right: vec![
                ToolbarItem2::Plugin("@seelen/tb-system-tray".into()),
                ToolbarItem2::Plugin("@seelen/tb-keyboard-selector".into()),
                ToolbarItem2::Plugin("@seelen/tb-bluetooth-popup".into()),
                ToolbarItem2::Plugin("@seelen/tb-network-popup".into()),
                ToolbarItem2::Plugin("@seelen/tb-media-popup".into()),
                ToolbarItem2::Plugin("@default/power".into()),
                ToolbarItem2::Plugin("@seelen/tb-notifications".into()),
                ToolbarItem2::Plugin("@seelen/tb-quick-settings".into()),
            ],
            ..Default::default()
        }
    }

    pub fn emit_toolbar_items(&self) -> Result<()> {
        emit_to_webviews(SeelenEvent::StateToolbarItemsChanged, &self.toolbar_items);
        Ok(())
    }

    fn _read_toolbar_items(&mut self) -> Result<()> {
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

    pub(super) fn read_toolbar_items(&mut self) {
        if let Err(err) = self._read_toolbar_items() {
            log::error!("Failed to read toolbar items: {err}");
            Self::show_corrupted_state_to_user(SEELEN_COMMON.toolbar_items_path());
        }
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
