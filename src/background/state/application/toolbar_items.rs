use std::{collections::HashSet, sync::LazyLock};

use parking_lot::Mutex;
use seelen_core::{
    resource::WidgetId,
    state::{ToolbarItem, ToolbarItem2, ToolbarState},
};

use crate::{
    error::{Result, ResultLogExt},
    utils::{atomic_write_file, constants::SEELEN_COMMON},
};

pub static TOOLBAR_ITEMS_MANAGER: LazyLock<ToolbarItemsManager> = LazyLock::new(|| {
    let manager = ToolbarItemsManager {
        items: Mutex::new(ToolbarState::default()),
    };
    manager.load().log_error();
    manager
});

pub struct ToolbarItemsManager {
    items: Mutex<ToolbarState>,
}

impl ToolbarItemsManager {
    pub fn get(&self) -> ToolbarState {
        self.items.lock().clone()
    }

    pub fn write(&self, mut items: ToolbarState) -> Result<()> {
        items.sanitize();
        let path = SEELEN_COMMON
            .widget_data_dir(&WidgetId::known_toolbar())
            .join("state.yml");
        atomic_write_file(&path, serde_yaml::to_string(&items)?.as_bytes())?;
        *self.items.lock() = items;
        Ok(())
    }

    pub fn load(&self) -> Result<()> {
        let path = SEELEN_COMMON
            .widget_data_dir(&WidgetId::known_toolbar())
            .join("state.yml");

        let items = if path.exists() {
            let mut items: ToolbarState = serde_yaml::from_str(&std::fs::read_to_string(&path)?)?;
            items.sanitize();
            items
        } else {
            let mut items = initial_items();
            items.sanitize();
            atomic_write_file(&path, serde_yaml::to_string(&items)?.as_bytes())?;
            items
        };

        *self.items.lock() = items;
        Ok(())
    }
}

fn initial_items() -> ToolbarState {
    ToolbarState {
        left: vec![
            ToolbarItem2::Plugin("@seelen/tb-user-menu".into()),
            ToolbarItem2::Inline(Box::new(ToolbarItem {
                template: "return \"|\"".into(),
                ..Default::default()
            })),
            ToolbarItem2::Plugin("@default/focused-app".into()),
            ToolbarItem2::Inline(Box::new(ToolbarItem {
                scopes: HashSet::from(["FocusedApp".to_owned()]),
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
