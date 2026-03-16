/* use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    state::{
        PinnedWegItemData, RelaunchArguments, WegItem, WegItemSubtype, WegItems,
        WegPinnedItemsVisibility,
    },
    system_state::MonitorId,
};
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    modules::{
        apps::application::{UserAppWinEvent, UserAppsManager, USER_APPS_MANAGER},
        start::application::StartMenuManager,
    },
    state::application::FULL_STATE,
    trace_lock,
    utils::icon_extractor::{request_icon_extraction_from_file, request_icon_extraction_from_umid},
    windows_api::{types::AppUserModelId, window::Window, MonitorEnumerator},
};

pub static SEELEN_WEG_STATE: LazyLock<Arc<Mutex<SeelenWegState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(SeelenWegState::new())));

#[derive(Debug, Clone)]
pub struct SeelenWegState {
    pub items: WegItems,
}

impl SeelenWegState {
    pub fn new() -> Self {
        let mut state = SeelenWegState {
            items: FULL_STATE.load().weg_items.clone(),
        };

        UserAppsManager::subscribe(|e| {
            if let UserAppWinEvent::Added(addr) = e {
                let mut guard = trace_lock!(SEELEN_WEG_STATE);
                if guard.add(&Window::from(addr)).unwrap_or(false) {
                    guard.emit_to_webview().log_error();
                }
            }
        });

        USER_APPS_MANAGER.interactable_windows.for_each(|w| {
            state.add(&Window::from(w.hwnd)).log_error();
        });

        state
    }


    pub fn iter_all(&self) -> impl Iterator<Item = &WegItem> {
        self.items
            .left
            .iter()
            .chain(self.items.center.iter())
            .chain(self.items.right.iter())
    }

    /// Adds a Temporal item for the given window's app if no matching item already exists.
    /// Returns `true` if a new item was added, `false` if one already existed.
    pub fn add(&mut self, window: &Window) -> Result<bool> {
        // we get the path of the creator in case of Application Frame Host, Web apps
        let mut path = match window.get_frame_creator() {
            Ok(None) => return Ok(false),
            Ok(Some(creator)) => creator.process().program_path()?,
            Err(_) => window.process().program_path()?,
        };

        let umid = window.app_user_model_id();
        let mut display_name = window
            .app_display_name()
            .unwrap_or_else(|_| String::from("Unknown"));

        let (relaunch_program, relaunch_args) = if let Some(umid) = &umid {
            match umid {
                AppUserModelId::Appx(umid) => {
                    // pre-extraction to avoid flickering on the ui
                    request_icon_extraction_from_umid(&AppUserModelId::Appx(umid.clone()));
                    (format!("shell:AppsFolder\\{umid}"), None)
                }
                AppUserModelId::PropertyStore(umid) => {
                    let start_menu_manager = StartMenuManager::instance();
                    let shortcut = start_menu_manager.get_by_file_umid(umid);

                    // some apps like librewolf don't have a shortcut with the same umid
                    if let Some(shortcut) = &shortcut {
                        // pre-extraction to avoid flickering on the ui
                        request_icon_extraction_from_umid(&AppUserModelId::PropertyStore(
                            umid.clone(),
                        ));
                        path = shortcut.path.clone();
                        display_name = path
                            .file_stem()
                            .unwrap_or_else(|| OsStr::new("Unknown"))
                            .to_string_lossy()
                            .to_string();
                    } else {
                        // pre-extraction to avoid flickering on the ui
                        request_icon_extraction_from_file(&path);
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
                        (format!("shell:AppsFolder\\{umid}"), None)
                    } else {
                        // process program path
                        (path.to_string_lossy().to_string(), None)
                    }
                }
            }
        } else {
            // pre-extraction to avoid flickering on the ui
            request_icon_extraction_from_file(&path);
            (path.to_string_lossy().to_string(), None)
        };

        let umid = umid.map(|umid| umid.to_string());
        let relaunch_args = relaunch_args.map(RelaunchArguments::String);

        // groups order documented on https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
        // group should be by umid, if not present then the groups are done by relaunch command
        // and in last case the groups are done by process id/path
        for item in self.iter_all() {
            match item {
                WegItem::Pinned(current) | WegItem::Temporal(current) => {
                    if (current.umid.is_some() && current.umid == umid)
                        || (current.relaunch_program.to_lowercase()
                            == relaunch_program.to_lowercase()
                            && current.relaunch_args == relaunch_args)
                    {
                        return Ok(false);
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
            relaunch_program,
            relaunch_args,
            relaunch_in: None,
            display_name,
            pin_disabled: window.prevent_pinning(),
        };
        self.items.center.push(WegItem::Temporal(data));
        Ok(true)
    }
}
 */
