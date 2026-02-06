use std::{
    collections::HashSet,
    fs::{create_dir_all, OpenOptions},
    io::Write,
};

use seelen_core::{
    resource::WidgetId,
    state::{ToolbarItem, ToolbarItem2, ToolbarJsScope, ToolbarState},
};

use crate::{error::Result, utils::constants::SEELEN_COMMON};

use super::FullState;

impl FullState {
    pub fn initial_toolbar_items() -> ToolbarState {
        ToolbarState {
            left: vec![
                ToolbarItem2::Plugin("@seelen/tb-user-menu".into()),
                ToolbarItem2::Inline(Box::new(ToolbarItem {
                    template: "return \"|\"".into(),
                    ..Default::default()
                })),
                ToolbarItem2::Plugin("@default/focused-app".into()),
                ToolbarItem2::Inline(Box::new(ToolbarItem {
                    scopes: HashSet::from([ToolbarJsScope::FocusedApp]),
                    template: "return focusedApp.title ? \"-\" : \"\"".into(),
                    ..Default::default()
                })),
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

    fn _read_toolbar_items(&mut self) -> Result<()> {
        let path = SEELEN_COMMON
            .widget_data_dir(&WidgetId::known_toolbar())
            .join("state.yml");

        if path.exists() {
            self.toolbar_items = serde_yaml::from_str(&std::fs::read_to_string(path)?)?;
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
            Self::show_corrupted_state_to_user(
                &SEELEN_COMMON.widget_data_dir(&WidgetId::known_toolbar()),
            );
        }
    }

    pub fn write_toolbar_items(&self, items: &ToolbarState) -> Result<()> {
        let dir = SEELEN_COMMON.widget_data_dir(&WidgetId::known_toolbar());
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
