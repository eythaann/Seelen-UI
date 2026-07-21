use std::{
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

use seelen_core::{state::AppExtraFlag, system_state::UserAppWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    WS_CHILD, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MINIMIZEBOX,
};

use crate::{
    hook::HookManager,
    modules::apps::application::{UserAppWinEvent, UserAppsManager, USER_APPS_MANAGER},
    utils::spawn_named_thread,
    windows_api::{
        event_window::IS_INTERACTIVE_SESSION,
        window::{event::WinEvent, Window},
        WindowEnumerator, WindowsApi,
    },
};

pub(super) fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl UserAppsManager {
    pub(super) fn init_listing_app_windows() -> Vec<UserAppWindow> {
        let mut initial = Vec::new();
        let _ = WindowEnumerator::new().for_each(|window| {
            if is_interactable_window(&window) {
                initial.push(window.to_serializable());
            }
        });

        // The enumerator yields windows in z-order (top-most first). Stamp
        // last_foreground_at so that clients sorting by that field preserve
        // z-order on first load. Each item gets now - index, keeping relative
        // ordering without blocking any future foreground update.
        let now = now_millis();
        for (i, w) in initial.iter_mut().enumerate() {
            w.last_foreground_at = now - i as i64;
        }

        HookManager::subscribe(|(event, window)| Self::on_win_event(event, window));

        spawn_named_thread("InteractableWindowsRevalidator", || loop {
            std::thread::sleep(std::time::Duration::from_millis(5000));

            // Pause when session is not interactive to reduce CPU usage
            if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
                continue;
            }

            Self::instance().interactable_windows.retain(|w| {
                let window = Window::from(w.hwnd);
                if window.is_interactable_and_not_hidden() {
                    true
                } else {
                    Self::send(UserAppWinEvent::Removed(window.address()));
                    false
                }
            });

            // Scan for windows that now qualify but were never tracked (e.g. style/frame
            // state was not settled at creation time).
            let _ = WindowEnumerator::new().for_each(|window| {
                if is_interactable_window(&window) && USER_APPS_MANAGER.add_win(&window) {
                    Self::send(UserAppWinEvent::Added(window.address()));
                }
            });
        });

        initial
    }

    fn on_win_event(event: WinEvent, window: Window) {
        let mut is_interactable = USER_APPS_MANAGER.contains_win(&window);

        match event {
            WinEvent::ObjectCreate | WinEvent::ObjectShow => {
                if !is_interactable
                    && is_interactable_window(&window)
                    && USER_APPS_MANAGER.add_win(&window)
                {
                    Self::send(UserAppWinEvent::Added(window.address()));
                }
            }
            WinEvent::ObjectNameChange
            | WinEvent::ObjectParentChange
            | WinEvent::ObjectUncloaked
            | WinEvent::ObjectStateChange => {
                let was_interactable = is_interactable;
                is_interactable = is_interactable_window(&window);
                match (was_interactable, is_interactable) {
                    (false, true) => {
                        if USER_APPS_MANAGER.add_win(&window) {
                            Self::send(UserAppWinEvent::Added(window.address()));
                        }
                    }
                    (true, false) => {
                        USER_APPS_MANAGER.remove_win(&window);
                        Self::send(UserAppWinEvent::Removed(window.address()));
                    }
                    _ => {}
                }

                // re-check for UWP apps that on creation starts without a parent
                if event == WinEvent::ObjectParentChange {
                    if let Some(parent) = window.parent() {
                        if parent.is_interactable_and_not_hidden()
                            && USER_APPS_MANAGER.add_win(&parent)
                        {
                            Self::send(UserAppWinEvent::Added(parent.address()));
                        }
                    }
                }
            }
            WinEvent::ObjectHide => {
                // UWP ApplicationFrameHosts are always hidden on minimize
                if is_interactable && !window.is_frame().unwrap_or(false) {
                    USER_APPS_MANAGER.remove_win(&window);
                    Self::send(UserAppWinEvent::Removed(window.address()));
                }
            }
            #[allow(clippy::collapsible_match)]
            WinEvent::ObjectDestroy => {
                if is_interactable {
                    USER_APPS_MANAGER.remove_win(&window);
                    Self::send(UserAppWinEvent::Removed(window.address()));
                }
            }
            _ => {}
        }

        // update cases on UserAppWindow
        if is_interactable {
            let mut changed = false;
            USER_APPS_MANAGER.interactable_windows.for_each(|entry| {
                if entry.hwnd == window.address() {
                    changed = Self::update_window_data(entry, event);
                }
            });

            if changed {
                Self::send(UserAppWinEvent::Updated(window.address()));
            }
        }
    }

    fn update_window_data(data: &mut UserAppWindow, event: WinEvent) -> bool {
        match event {
            WinEvent::ObjectNameChange => {
                let window = Window::from(data.hwnd);
                data.title = window.title();
                true
            }
            WinEvent::SynDebouncedRectChange => {
                let window = Window::from(data.hwnd);
                data.is_zoomed = window.is_maximized();
                data.is_fullscreen = window.is_fullscreen();
                data.rect = window.inner_rect().ok();
                data.monitor = window.monitor().stable_id().unwrap_or_default();
                true
            }
            WinEvent::SystemForeground => {
                let now = now_millis();
                data.last_foreground_at = now;
                true
            }
            WinEvent::SystemMinimizeStart => {
                data.is_iconic = true;
                true
            }
            WinEvent::SystemMinimizeEnd => {
                data.is_iconic = false;
                true
            }
            _ => false,
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

    match to_validate.get_app_config() {
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
