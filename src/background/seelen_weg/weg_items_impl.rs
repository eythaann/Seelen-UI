use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::state::{PinnedWegItemData, WegAppGroupItem, WegItem, WegItems};
use std::sync::Arc;

use crate::{error_handler::Result, state::application::FULL_STATE, windows_api::window::Window};

use super::icon_extractor::{extract_and_save_icon_from_file, extract_and_save_icon_umid};

lazy_static! {
    pub static ref WEG_ITEMS_IMPL: Arc<Mutex<WegItemsImpl>> =
        Arc::new(Mutex::new(WegItemsImpl::default()));
}

#[derive(Debug, Default)]
pub struct WegItemsImpl {
    items: WegItems,
}

fn item_contains_window(item: &WegItem, searching: isize) -> bool {
    match item {
        WegItem::Pinned(data) | WegItem::Temporal(data) => {
            data.windows.iter().any(|w| w.handle == searching)
        }
        _ => false,
    }
}

impl WegItemsImpl {
    pub fn new() -> Self {
        WegItemsImpl {
            items: FULL_STATE.load().weg_items.clone(),
        }
    }

    pub fn iter_all(&self) -> impl Iterator<Item = &WegItem> {
        self.items
            .left
            .iter()
            .chain(self.items.center.iter())
            .chain(self.items.right.iter())
    }

    pub fn iter_all_mut(&mut self) -> impl Iterator<Item = &mut WegItem> {
        self.items
            .left
            .iter_mut()
            .chain(self.items.center.iter_mut())
            .chain(self.items.right.iter_mut())
    }

    pub fn contains(&self, window: &Window) -> bool {
        let searching = window.address();
        self.iter_all()
            .any(|item| item_contains_window(item, searching))
    }

    pub fn add(&mut self, window: &Window) -> Result<()> {
        if self.contains(window) {
            return Ok(());
        }

        let creator = match window.get_frame_creator() {
            Ok(None) => return Ok(()),
            Ok(Some(creator)) => creator,
            Err(_) => *window,
        };

        let path = creator.exe()?;
        let execution_command = if let Ok(umid) = creator.process().package_app_user_model_id() {
            let _ = extract_and_save_icon_umid(&umid);
            format!("shell:AppsFolder\\{umid}")
        } else {
            let _ = extract_and_save_icon_from_file(&path);
            path.to_string_lossy().to_string()
        };

        for item in self.iter_all_mut() {
            match item {
                WegItem::Pinned(data) | WegItem::Temporal(data) => {
                    if data.execution_command == execution_command {
                        data.windows.push(WegAppGroupItem {
                            title: window.title(),
                            handle: window.address(),
                        });
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        let data = PinnedWegItemData {
            path,
            execution_command,
            is_dir: false,
            windows: vec![WegAppGroupItem {
                title: window.title(),
                handle: window.address(),
            }],
        };
        self.items.center.push(WegItem::Temporal(data));
        Ok(())
    }

    pub fn remove(&mut self, window: &Window) {
        let searching = window.address();
        self.iter_all_mut().for_each(|item| match item {
            WegItem::Pinned(data) | WegItem::Temporal(data) => {
                data.windows.retain(|w| w.handle != searching);
            }
            _ => {}
        });
        self.items.sanitize();
    }

    pub fn get(&self) -> WegItems {
        self.items.clone()
    }
}
