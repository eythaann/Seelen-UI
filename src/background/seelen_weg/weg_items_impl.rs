use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    state::{
        PinnedWegItemData, RelaunchArguments, WegAppGroupItem, WegItem, WegItemSubtype, WegItems,
        WegPinnedItemsVisibility, WegTemporalItemsVisibility,
    },
};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf, sync::Arc};
use tauri::Emitter;

use crate::{
    error_handler::Result,
    modules::start::application::START_MENU_MANAGER,
    seelen::get_app_handle,
    state::application::FULL_STATE,
    utils::icon_extractor::{extract_and_save_icon_from_file, extract_and_save_icon_umid},
    windows_api::{types::AppUserModelId, window::Window, MonitorEnumerator},
};

use super::SeelenWeg;

lazy_static! {
    pub static ref WEG_ITEMS_IMPL: Arc<Mutex<WegItemsImpl>> =
        Arc::new(Mutex::new(WegItemsImpl::new()));
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

fn get_parts_of_inline_command(cmd: &str) -> (String, Option<String>) {
    let start_double_quoted = cmd.starts_with("\"");
    if start_double_quoted || cmd.starts_with("'") {
        let delimiter = if start_double_quoted { '"' } else { '\'' };
        let mut parts = cmd.split(['"', '\'']).filter(|s| !s.is_empty());

        let program = parts.next().unwrap_or_default().trim().to_owned();
        let args = cmd
            .trim_start_matches(&format!("{delimiter}{program}{delimiter}"))
            .trim()
            .to_owned();
        return (program, if args.is_empty() { None } else { Some(args) });
    }

    let cmd_as_path = PathBuf::from(cmd);
    if cmd_as_path.exists() {
        let program = cmd_as_path.to_string_lossy().to_string();
        return (program, None);
    }

    let mut parts = cmd.split(" ").filter(|s| !s.is_empty());
    let program = parts.next().unwrap_or_default().trim().to_owned();
    let args = cmd.trim_start_matches(&program).trim().to_owned();
    (program, if args.is_empty() { None } else { Some(args) })
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

    #[allow(deprecated)]
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

        let umid = window.app_user_model_id();
        let mut display_name = window
            .app_display_name()
            .unwrap_or_else(|_| String::from("Unknown"));

        let (relaunch_program, relaunch_args) = if let Some(umid) = &umid {
            // pre-extraction to avoid flickering on the ui
            let _ = extract_and_save_icon_umid(umid);
            match umid {
                AppUserModelId::Appx(umid) => (
                    "C:\\Windows\\explorer.exe".to_owned(),
                    Some(format!("shell:AppsFolder\\{umid}")),
                ),
                AppUserModelId::PropertyStore(umid) => {
                    let shortcut = START_MENU_MANAGER
                        .load()
                        .search_shortcut_with_same_umid(umid);

                    // some apps like librewolf don't have a shortcut with the same umid
                    if let Some(shortcut) = &shortcut {
                        path = shortcut.clone();
                        display_name = path
                            .file_stem()
                            .unwrap_or_else(|| OsStr::new("Unknown"))
                            .to_string_lossy()
                            .to_string();
                    } else {
                        // pre-extraction to avoid flickering on the ui
                        let _ = extract_and_save_icon_from_file(&path);
                    }

                    // System.AppUserModel.RelaunchCommand and System.AppUserModel.RelaunchDisplayNameResource
                    // must always be set together. If one of those properties is not set, then neither is used.
                    // https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchcommand
                    if let (Some(relaunch_command), Some(relaunch_display_name)) =
                        (window.relaunch_command(), window.relaunch_display_name())
                    {
                        display_name = relaunch_display_name;
                        get_parts_of_inline_command(&relaunch_command)
                    } else if shortcut.is_some() {
                        (
                            "C:\\Windows\\explorer.exe".to_owned(),
                            Some(format!("shell:AppsFolder\\{umid}")),
                        )
                    } else {
                        // process program path
                        (path.to_string_lossy().to_string(), None)
                    }
                }
            }
        } else {
            // pre-extraction to avoid flickering on the ui
            let _ = extract_and_save_icon_from_file(&path);
            (path.to_string_lossy().to_string(), None)
        };

        let umid = umid.map(|umid| umid.to_string());
        let relaunch_args = relaunch_args.map(RelaunchArguments::String);
        // groups order documented on https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
        // group should be by umid, if not present then the groups are done by relaunch command
        // and in last case the groups are done by process id/path
        for item in self.iter_all_mut() {
            match item {
                WegItem::Pinned(current) | WegItem::Temporal(current) => {
                    if (current.umid.is_some() && current.umid == umid)
                        || (current.relaunch_program.to_lowercase()
                            == relaunch_program.to_lowercase()
                            && current.relaunch_args == relaunch_args)
                    {
                        current.windows.push(WegAppGroupItem {
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
            subtype: WegItemSubtype::App,
            umid,
            path,
            relaunch_command: None,
            relaunch_program,
            relaunch_args,
            relaunch_in: None,
            display_name,
            is_dir: false,
            windows: vec![WegAppGroupItem {
                title: window.title(),
                handle: window.address(),
            }],
            pin_disabled: window.prevent_pinning(),
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
