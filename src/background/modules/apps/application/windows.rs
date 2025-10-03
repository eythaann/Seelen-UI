use seelen_core::system_state::UserAppWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_MINIMIZEBOX,
};

use crate::{
    hook::HookManager,
    modules::apps::application::{UserAppsEvent, UserAppsManager, USER_APPS_MANAGER},
    windows_api::{
        window::{event::WinEvent, Window},
        WindowEnumerator, WindowsApi,
    },
};

impl UserAppsManager {
    pub(super) fn init_listing_app_windows() -> Vec<UserAppWindow> {
        let mut initial = Vec::new();
        let _ = WindowEnumerator::new().for_each(|window| {
            if is_interactable_and_not_hidden(&window) {
                initial.push(window.to_serializable());
            }
        });

        HookManager::subscribe(|(event, window)| Self::on_win_event(event, window));
        initial
    }

    fn on_win_event(event: WinEvent, window: Window) {
        let mut is_interactable = USER_APPS_MANAGER.contains_win(&window);

        match event {
            WinEvent::ObjectCreate | WinEvent::ObjectShow => {
                if !is_interactable && is_interactable_and_not_hidden(&window) {
                    USER_APPS_MANAGER.add_win(&window);
                    Self::send(UserAppsEvent::WinAdded(window.address()));
                }
            }
            WinEvent::ObjectNameChange => {
                let was_interactable = is_interactable;
                is_interactable = is_interactable_and_not_hidden(&window);
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
            }
            WinEvent::ObjectParentChange => {
                // re-check for UWP apps that on creation starts without a parent
                if let Some(parent) = window.parent() {
                    if !USER_APPS_MANAGER.contains_win(&parent)
                        && parent.is_interactable_and_not_hidden()
                    {
                        USER_APPS_MANAGER.add_win(&parent);
                        Self::send(UserAppsEvent::WinAdded(parent.address()));
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
pub fn is_interactable_and_not_hidden(window: &Window) -> bool {
    if !window.is_visible() {
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

    // Discard unminimizable windows
    let style = WindowsApi::get_styles(window.hwnd());
    if !style.contains(WS_MINIMIZEBOX) {
        return false;
    }

    // Discard layered windows without WS_EX_APPWINDOW
    let ex_style = WindowsApi::get_ex_styles(window.hwnd());
    if (ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE))
        && !ex_style.contains(WS_EX_APPWINDOW)
    {
        return false;
    }

    // unmanageable window, these probably are system processes
    if window.process().open_limited_handle().is_err() {
        return false;
    }

    if let Ok(frame_creator) = window.get_frame_creator() {
        if frame_creator.is_none() {
            return false;
        }
    }

    if window.process().is_frozen().unwrap_or(false) {
        return false;
    }

    true
}

/* fn notify_changed_to_ui() {
    let items = get_user_app_windows();
    get_app_handle()
        .emit(SeelenEvent::UserAppWindowsChanged, items)
        .wrap_error()
        .log_error();
}
 */
