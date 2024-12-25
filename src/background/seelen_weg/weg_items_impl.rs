use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    state::{PinnedWegItemData, WegAppGroupItem, WegItem, WegItems},
};
use std::sync::Arc;
use tauri::Emitter;

use crate::{
    error_handler::Result, seelen::get_app_handle, state::application::FULL_STATE,
    windows_api::window::Window,
};

use super::icon_extractor::{extract_and_save_icon_from_file, extract_and_save_icon_umid};

lazy_static! {
    pub static ref WEG_ITEMS_IMPL: Arc<Mutex<WegItemsImpl>> =
        Arc::new(Mutex::new(WegItemsImpl::new()));
}

#[derive(Debug)]
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

    pub fn on_stored_changed(&mut self, stored: WegItems) -> Result<()> {
        let mut handles = vec![];
        for item in self.iter_all() {
            if let WegItem::Pinned(data) | WegItem::Temporal(data) = item {
                for w in &data.windows {
                    handles.push(w.handle);
                }
            }
        }
        self.items = stored;
        for handle in handles {
            self.add(&Window::from(handle))?;
        }
        get_app_handle().emit(SeelenEvent::WegInstanceChanged, self.get())?;
        Ok(())
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

        // we get the path of the creator in case of Application Frame Host, Web apps
        let mut path = match window.get_frame_creator() {
            Ok(None) => return Ok(()),
            Ok(Some(creator)) => creator.exe()?,
            Err(_) => window.exe()?,
        };

        let package_id = window.process().package_app_user_model_id().ok();
        let assigned_umid = window.app_user_model_id();

        let relaunch_command = if let Some(umid) = package_id.as_ref() {
            let _ = extract_and_save_icon_umid(umid);
            format!("\"explorer.exe\" shell:AppsFolder\\{umid}")
        } else if let Some(umid) = assigned_umid.as_ref() {
            let shortcut = Window::search_shortcut_with_same_umid(umid);
            if let Some(shortcut) = shortcut {
                path = shortcut.clone();
                let _ = extract_and_save_icon_from_file(&path);
            }
            let _ = extract_and_save_icon_from_file(&path);
            window
                .relaunch_command()
                .unwrap_or(format!("\"explorer.exe\" shell:AppsFolder\\{umid}"))
        } else {
            let _ = extract_and_save_icon_from_file(&path);
            path.to_string_lossy().to_string()
        };

        // groups order documented on https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
        // group should be by umid, if not present then the groups are done by relaunch command
        // and in last case the groups are done by process id/path
        for item in self.iter_all_mut() {
            match item {
                WegItem::Pinned(data) | WegItem::Temporal(data) => {
                    if package_id.as_ref().is_some_and(|umid| umid == &data.id) {
                        data.windows.push(WegAppGroupItem {
                            title: window.title(),
                            handle: window.address(),
                        });
                        return Ok(());
                    }

                    if assigned_umid.as_ref().is_some_and(|umid| umid == &data.id) {
                        data.windows.push(WegAppGroupItem {
                            title: window.title(),
                            handle: window.address(),
                        });
                        return Ok(());
                    }

                    if data.relaunch_command == relaunch_command {
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
            id: uuid::Uuid::new_v4().to_string(),
            umid: package_id.or(assigned_umid),
            path,
            relaunch_command,
            display_name: window
                .app_display_name()
                .unwrap_or_else(|_| "Unkown".to_string()),
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
