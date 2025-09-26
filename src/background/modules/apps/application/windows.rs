use seelen_core::{state::AppExtraFlag, system_state::UserAppWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

use crate::{
    hook::HookManager,
    modules::apps::application::{UserAppsEvent, UserAppsManager, USER_APPS_MANAGER},
    state::application::FULL_STATE,
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
        let mut is_interactable = USER_APPS_MANAGER.contains_win(&window.address());

        match event {
            WinEvent::ObjectCreate => {
                if is_interactable_and_not_hidden(&window) {
                    USER_APPS_MANAGER
                        .interactable_windows
                        .push(window.to_serializable());
                    Self::send(UserAppsEvent::WinAdded);
                }
            }
            WinEvent::ObjectShow
            | WinEvent::ObjectUncloaked
            | WinEvent::ObjectNameChange
            | WinEvent::ObjectParentChange => {
                let was_interactable = is_interactable;
                is_interactable = is_interactable_and_not_hidden(&window);
                match (was_interactable, is_interactable) {
                    (false, true) => {
                        USER_APPS_MANAGER
                            .interactable_windows
                            .push(window.to_serializable());
                        Self::send(UserAppsEvent::WinAdded);
                    }
                    (true, false) => {
                        let addr = window.address();
                        USER_APPS_MANAGER
                            .interactable_windows
                            .retain(|w| w.hwnd != addr);
                        Self::send(UserAppsEvent::WinRemoved);
                    }
                    _ => {}
                }
            }
            WinEvent::ObjectDestroy => {
                if is_interactable {
                    USER_APPS_MANAGER
                        .interactable_windows
                        .retain(|w| w.hwnd != window.address());
                    Self::send(UserAppsEvent::WinRemoved);
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
            Self::send(UserAppsEvent::WinUpdated);
        }
    }
}

/// The idea with this module is contain all the logic under the filteriong of windows
/// that can be considered as applications windows, it means windows that are interactable
/// for the users.
///
/// As windows properties can change, this should be reevaluated on every change.
pub fn is_interactable_and_not_hidden(window: &Window) -> bool {
    // unmanageable window
    if window.process().open_limited_handle().is_err() {
        return false;
    }

    if !window.is_visible() || window.parent().is_some() {
        return false;
    }

    // ignore windows without a title or that start with a dot
    // this is a seelen ui behavior, not present on other desktop environments
    let title = window.title();
    if title.is_empty() || title.starts_with('.') {
        return false;
    }

    // this class is used for edge tabs to be shown as independent windows on alt + tab
    // this only applies when the new tab is created it is binded to explorer.exe for some reason
    // maybe we can search/learn more about edge tabs later.
    // fix: https://github.com/eythaann/Seelen-UI/issues/83
    if window.class() == "Windows.Internal.Shell.TabProxyWindow" {
        return false;
    }

    let ex_style = WindowsApi::get_ex_styles(window.hwnd());
    if (ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE))
        && !ex_style.contains(WS_EX_APPWINDOW)
    {
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

    // I don't like to determine if a window is real filtering by this configs, but will be here
    // as a workaround in meantime we find a way to filter better, as native taskbar does.
    if let Some(config) = FULL_STATE.load().get_app_config_by_window(window.hwnd()) {
        if config.options.contains(&AppExtraFlag::Hidden) {
            return false;
        }
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
