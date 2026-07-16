use std::sync::LazyLock;

use parking_lot::Mutex;
use seelen_core::{
    resource::WidgetId,
    state::{WegItem, WegItemData, WegItems},
};
use uuid::Uuid;

use crate::{
    error::{Result, ResultLogExt},
    modules::apps::application::msix::MsixAppsManager,
    utils::{atomic_write_file, constants::SEELEN_COMMON},
};

pub static WEG_ITEMS_MANAGER: LazyLock<WegItemsManager> = LazyLock::new(|| {
    let manager = WegItemsManager {
        items: Mutex::new(WegItems::default()),
    };
    manager.load().log_error();
    manager
});

pub struct WegItemsManager {
    items: Mutex<WegItems>,
}

impl WegItemsManager {
    pub fn get(&self) -> WegItems {
        self.items.lock().clone()
    }

    pub fn write(&self, mut items: WegItems) -> Result<()> {
        items.sanitize();
        let path = SEELEN_COMMON
            .widget_data_dir(&WidgetId::known_weg())
            .join("state.yml");
        atomic_write_file(&path, serde_yaml::to_string(&items)?.as_bytes())?;
        *self.items.lock() = items;
        Ok(())
    }

    pub fn load(&self) -> Result<()> {
        let path = SEELEN_COMMON
            .widget_data_dir(&WidgetId::known_weg())
            .join("state.yml");

        let items = if path.exists() {
            let mut items: WegItems = serde_yaml::from_str(&std::fs::read_to_string(&path)?)?;
            items.sanitize();
            resolve_msix_paths(&mut items);
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

fn initial_items() -> WegItems {
    WegItems {
        is_reorder_disabled: false,
        left: vec![
            WegItem::Plugin {
                id: Uuid::new_v4(),
                plugin: "@default/weg-start-menu".into(),
            },
            WegItem::Plugin {
                id: Uuid::new_v4(),
                plugin: "@default/weg-show-desktop".into(),
            },
        ],
        center: vec![WegItem::AppOrFile(WegItemData {
            id: Uuid::new_v4(),
            umid: None,
            path: "C:\\Windows\\explorer.exe".into(),
            display_name: t!("file_explorer").to_string(),
            pinned: true,
            prevent_pinning: false,
            relaunch: None,
        })],
        right: vec![
            WegItem::Plugin {
                id: Uuid::new_v4(),
                plugin: "@default/weg-trash-bin".into(),
            },
            WegItem::Media { id: Uuid::new_v4() },
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

fn resolve_msix_paths(weg_items: &mut WegItems) {
    let WegItems {
        left,
        center,
        right,
        ..
    } = weg_items;
    std::thread::scope(|s| {
        s.spawn(|| update_weg_items_paths(left));
        s.spawn(|| update_weg_items_paths(center));
        s.spawn(|| update_weg_items_paths(right));
    });
}
