use seelen_core::{state::AppExtraFlag, system_state::UserAppWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    WS_CHILD, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MINIMIZEBOX,
};

use crate::{
    hook::HookManager,
    modules::apps::application::{UserAppsEvent, UserAppsManager, USER_APPS_MANAGER},
    state::application::FULL_STATE,
    utils::spawn_named_thread,
    windows_api::{
        window::{event::WinEvent, Window},
        WindowEnumerator, WindowsApi,
    },
};

impl UserAppsManager {
    pub(super) fn init_listing_app_windows() -> Vec<UserAppWindow> {
        let mut initial = Vec::new();
        let _ = WindowEnumerator::new().for_each(|window| {
            if is_interactable_window(&window) {
                initial.push(window.to_serializable());
            }
        });

        HookManager::subscribe(|(event, window)| Self::on_win_event(event, window));

        spawn_named_thread("InteractableWindowsRevalidator", || loop {
            std::thread::sleep(std::time::Duration::from_millis(2000));
            Self::instance().interactable_windows.retain(|w| {
                let window = Window::from(w.hwnd);
                if window.is_interactable_and_not_hidden() {
                    true
                } else {
                    Self::send(UserAppsEvent::WinRemoved(window.address()));
                    false
                }
            });
        });

        initial
    }

    fn on_win_event(event: WinEvent, window: Window) {
        let mut is_interactable = USER_APPS_MANAGER.contains_win(&window);

        match event {
            WinEvent::ObjectCreate | WinEvent::ObjectShow => {
                if !is_interactable && is_interactable_window(&window) {
                    USER_APPS_MANAGER.add_win(&window);
                    Self::send(UserAppsEvent::WinAdded(window.address()));
                }
            }
            WinEvent::ObjectNameChange | WinEvent::ObjectParentChange => {
                let was_interactable = is_interactable;
                is_interactable = is_interactable_window(&window);
                match (was_interactable, is_interactable) {
                    (false, true) => {
                        USER_APPS_MANAGER.add_win(&window);
                        Self::send(UserAppsEvent::WinAdded(window.address()));
                    }
                    (true, false) => {
                        USER_APPS_MANAGER.remove_win(&window);
                        Self::send(UserAppsEvent::WinRemoved(window.address()));
                    }
                    _ => {}
                }

                // re-check for UWP apps that on creation starts without a parent
                if event == WinEvent::ObjectParentChange {
                    if let Some(parent) = window.parent() {
                        if !USER_APPS_MANAGER.contains_win(&parent)
                            && parent.is_interactable_and_not_hidden()
                        {
                            USER_APPS_MANAGER.add_win(&parent);
                            Self::send(UserAppsEvent::WinAdded(parent.address()));
                        }
                    }
                }
            }
            WinEvent::ObjectHide => {
                // UWP ApplicationFrameHosts are always hidden on minimize
                if is_interactable && !window.is_frame().unwrap_or(false) {
                    USER_APPS_MANAGER.remove_win(&window);
                    Self::send(UserAppsEvent::WinRemoved(window.address()));
                }
            }
            WinEvent::ObjectDestroy => {
                if is_interactable {
                    USER_APPS_MANAGER.remove_win(&window);
                    Self::send(UserAppsEvent::WinRemoved(window.address()));
                }
            }
            _ => {}
        }

        // update cases on UserAppWindow
        if is_interactable
            && matches!(
                event,
                WinEvent::ObjectNameChange
                    | WinEvent::SystemMinimizeStart
                    | WinEvent::SystemMinimizeEnd
                    | WinEvent::SyntheticMaximizeStart
                    | WinEvent::SyntheticMaximizeEnd
                    | WinEvent::SyntheticFullscreenStart
                    | WinEvent::SyntheticFullscreenEnd
                    | WinEvent::SyntheticMonitorChanged
            )
        {
            USER_APPS_MANAGER.interactable_windows.for_each(|w| {
                if w.hwnd == window.address() {
                    *w = window.to_serializable();
                }
            });
            Self::send(UserAppsEvent::WinUpdated(window.address()));
        }
    }
}

/// The idea with this module is contain all the logic under the filteriong of windows
/// that can be considered as applications windows, it means windows that are interactable
/// for the users.
///
/// As windows properties can change, this should be reevaluated on every change.
pub fn is_interactable_window(window: &Window) -> bool {
    // It must be a visible Window and not cloaked
    if !window.is_window() || !window.is_visible() || window.is_cloaked() {
        return false;
    }

    // ignore windows without a title, these are not intended to be shown to users (comonly are invisible windows)
    let title = window.title();
    if title.is_empty() {
        return false;
    }

    // this class is used for edge tabs to be shown as independent windows on alt + tab
    // this only applies when the new tab is created it is binded to explorer.exe for some reason
    // maybe we can search/learn more about edge tabs later.
    // fix: https://github.com/eythaann/Seelen-UI/issues/83
    if window.class() == "Windows.Internal.Shell.TabProxyWindow" {
        return false;
    }

    let style = WindowsApi::get_styles(window.hwnd());
    let ex_style = WindowsApi::get_ex_styles(window.hwnd());

    if !ex_style.contains(WS_EX_APPWINDOW) {
        // It must not be owned by another window
        if style.contains(WS_CHILD) || window.owner().is_some() {
            return false;
        }

        // Discard tool windows without WS_EX_APPWINDOW
        if ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE) {
            return false;
        }
    }

    let process = window.process();
    // unmanageable window, these probably are system processes
    if process.open_limited_handle().is_err() {
        return false;
    }

    // Internal behaviour for seelen ui widgets:
    // Discard unminimizable windows (they have no caption/title bar)
    if !style.contains(WS_MINIMIZEBOX) && process.is_seelen() {
        return false;
    }

    if process.is_frozen().unwrap_or(false) {
        return false;
    }

    let to_validate = match window.get_frame_creator() {
        Ok(None) => return false, // not found
        Ok(Some(creator)) => creator,
        Err(_) => *window, // window is not a frame
    };

    let guard = FULL_STATE.load();
    match guard.get_app_config_by_window(to_validate.hwnd()) {
        Ok(Some(config)) => {
            if config.options.contains(&AppExtraFlag::NoInteractive) {
                return false;
            }
        }
        Ok(_) => {}
        Err(err) => {
            log::error!("Error getting app config: {err}");
            return false;
        }
    }

    true
}
