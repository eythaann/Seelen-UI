use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    state::{
        PinnedWegItemData, WegAppGroupItem, WegItem, WegItems, WegPinnedItemsVisibility,
        WegTemporalItemsVisibility,
    },
};
use std::{collections::HashMap, ffi::OsStr, sync::Arc};
use tauri::Emitter;

use crate::{
    error_handler::Result,
    log_error,
    modules::start::application::START_MENU_MANAGER,
    seelen::get_app_handle,
    state::application::FULL_STATE,
    windows_api::{window::Window, MonitorEnumerator, WindowsApi},
};

use super::{
    icon_extractor::{extract_and_save_icon_from_file, extract_and_save_icon_umid},
    SeelenWeg,
};

lazy_static! {
    pub static ref WEG_ITEMS_IMPL: Arc<Mutex<WegItemsImpl>> =
        Arc::new(Mutex::new(WegItemsImpl::new()));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShouldGetInfoFrom {
    Package(String),
    WindowPropertyStore(String),
    Process,
}

#[derive(Debug, Clone)]
pub struct WegItemsImpl {
    items: WegItems,
    pre_state: Option<HashMap<String, WegItems>>,
}

fn item_contains_window(item: &WegItem, searching: isize) -> bool {
    match item {
        WegItem::Pinned(data) | WegItem::Temporal(data) => {
            data.windows.iter().any(|w| w.handle == searching)
        }
        _ => false,
    }
}

fn temporalise_collection(source: &Vec<WegItem>) -> Vec<WegItem> {
    let mut items = vec![];
    for item in source {
        match item {
            WegItem::Temporal(pinned_weg_item_data) => {
                let mut cloned = pinned_weg_item_data.clone();
                cloned.set_pin_disabled(true);
                items.push(WegItem::Temporal(cloned))
            }
            WegItem::Pinned(pinned_weg_item_data) => {
                let mut cloned = pinned_weg_item_data.clone();
                cloned.set_pin_disabled(true);
                items.push(WegItem::Temporal(cloned))
            }
            WegItem::Separator { id: _ }
            | WegItem::Media { id: _ }
            | WegItem::StartMenu { id: _ } => {}
        }
    }

    items
}

fn temporalise(items: &mut WegItems) {
    items.left = temporalise_collection(&items.left);
    items.center = temporalise_collection(&items.center);
    items.right = temporalise_collection(&items.right);
}

impl WegItemsImpl {
    pub fn new() -> Self {
        WegItemsImpl {
            items: FULL_STATE.load().weg_items.clone(),
            pre_state: None,
        }
    }

    pub fn emit_to_webview(&mut self) -> Result<()> {
        let handle = get_app_handle();
        let current_state = self.get_filtered_by_monitor().ok();

        if current_state != self.pre_state {
            if let Some(items) = &current_state {
                for (monitor_id, items) in items {
                    handle.emit_to(
                        SeelenWeg::get_label(monitor_id),
                        SeelenEvent::WegInstanceChanged,
                        items.clone(),
                    )?;
                }
            }

            self.pre_state = current_state;
        }

        Ok(())
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
        self.emit_to_webview()?;
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
            Ok(Some(creator)) => creator.process().program_path()?,
            Err(_) => window.process().program_path()?,
        };

        let umid = window
            .process()
            .package_app_user_model_id()
            .ok()
            .or_else(|| window.app_user_model_id());

        let get_info_from = match &umid {
            Some(umid) => {
                let _ = extract_and_save_icon_umid(umid);
                if WindowsApi::is_uwp_package_id(umid) {
                    ShouldGetInfoFrom::Package(umid.clone())
                } else {
                    ShouldGetInfoFrom::WindowPropertyStore(umid.clone())
                }
            }
            None => ShouldGetInfoFrom::Process,
        };

        let mut pin_disabled = false;
        let mut display_name = window
            .app_display_name()
            .unwrap_or_else(|_| String::from("Unknown"));

        let relaunch_command;
        match get_info_from {
            ShouldGetInfoFrom::Package(umid) => {
                display_name = WindowsApi::get_uwp_app_info(&umid)?
                    .DisplayInfo()?
                    .DisplayName()?
                    .to_string_lossy();
                relaunch_command = format!("\"explorer.exe\" shell:AppsFolder\\{umid}");
            }
            ShouldGetInfoFrom::WindowPropertyStore(umid) => {
                let shortcut = START_MENU_MANAGER
                    .load()
                    .search_shortcut_with_same_umid(&umid);
                if let Some(shortcut) = shortcut {
                    path = shortcut.clone();
                    display_name = path
                        .file_stem()
                        .unwrap_or_else(|| OsStr::new("Unknown"))
                        .to_string_lossy()
                        .to_string();
                } else {
                    log_error!(extract_and_save_icon_from_file(&path));
                }
                // System.AppUserModel.RelaunchCommand and System.AppUserModel.RelaunchDisplayNameResource
                // must always be set together. If one of those properties is not set, then neither is used.
                // https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchcommand
                if let (Some(win_relaunch_command), Some(relaunch_display_name)) =
                    (window.relaunch_command(), window.relaunch_display_name())
                {
                    relaunch_command = win_relaunch_command;
                    display_name = relaunch_display_name;
                } else {
                    pin_disabled = window.prevent_pinning();
                    relaunch_command = path.to_string_lossy().to_string();
                }
            }
            ShouldGetInfoFrom::Process => {
                log_error!(extract_and_save_icon_from_file(&path));
                relaunch_command = path.to_string_lossy().to_string();
            }
        };

        // groups order documented on https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
        // group should be by umid, if not present then the groups are done by relaunch command
        // and in last case the groups are done by process id/path
        for item in self.iter_all_mut() {
            match item {
                WegItem::Pinned(data) | WegItem::Temporal(data) => {
                    if data.umid.is_some() && umid == data.umid {
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
            umid,
            path,
            relaunch_command,
            display_name,
            is_dir: false,
            windows: vec![WegAppGroupItem {
                title: window.title(),
                handle: window.address(),
            }],
            pin_disabled,
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

    pub fn update_window(&mut self, window: &Window) {
        let searching = window.address();
        for item in self.iter_all_mut() {
            if let WegItem::Pinned(data) | WegItem::Temporal(data) = item {
                let maybe_window = data.windows.iter_mut().find(|w| w.handle == searching);
                if let Some(app_window) = maybe_window {
                    app_window.title = window.title();
                    break;
                }
            }
        }
    }

    fn filter_by_monitor(&mut self, monitor_id: &str) {
        for item in self.iter_all_mut() {
            match item {
                WegItem::Pinned(data) | WegItem::Temporal(data) => {
                    data.windows.retain(|w| {
                        let window = Window::from(w.handle);
                        window
                            .monitor()
                            .device_id()
                            .is_ok_and(|id| id == monitor_id)
                    });
                }
                _ => {}
            }
        }
    }

    pub fn get(&self) -> WegItems {
        self.items.clone()
    }

    pub fn get_filtered_by_monitor(&self) -> Result<HashMap<String, WegItems>> {
        let mut result = HashMap::new();
        let state = FULL_STATE.load();

        for monitor in MonitorEnumerator::get_all_v2()? {
            let monitor_id = monitor.device_id()?;
            if !state.is_weg_enabled_on_monitor(&monitor_id) {
                continue;
            }
            let temporal_mode = state.get_weg_temporal_item_visibility(&monitor_id);
            let pinned_mode = state.get_weg_pinned_item_visibility(&monitor_id);
            let pinned_visible = match pinned_mode {
                WegPinnedItemsVisibility::Always => true,
                WegPinnedItemsVisibility::WhenPrimary => monitor.is_primary(),
            };

            match temporal_mode {
                WegTemporalItemsVisibility::All => {
                    let mut items = self.items.clone();
                    if !pinned_visible {
                        temporalise(&mut items);
                        items.sanitize();
                    }
                    result.insert(monitor_id, items);
                }
                WegTemporalItemsVisibility::OnMonitor => {
                    let mut weg_items = self.clone();
                    weg_items.filter_by_monitor(&monitor_id);
                    if !pinned_visible {
                        temporalise(&mut weg_items.items);
                    }
                    weg_items.items.sanitize();
                    result.insert(monitor_id, weg_items.items);
                }
            }
        }

        Ok(result)
    }
}
