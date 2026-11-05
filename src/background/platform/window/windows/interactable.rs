use seelen_core::state::AppExtraFlag;
use windows::Win32::UI::WindowsAndMessaging::{
    WS_CHILD, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

use crate::{
    platform::{
        process::WindowsProcessExt,
        window::{Window, WindowsWindowExt},
    },
    windows_api::WindowsApi,
};

/// The idea with this module is to contain all the logic for filtering the windows
/// that can be considered as application windows, meaning windows that are interactable
/// for the users.
///
/// As window properties can change, this should be reevaluated on every change.
pub(in crate::platform::window) fn is_interactable_window(window: &Window) -> bool {
    // It must be a visible window and not cloaked. Cloaks done by our own buffered animations
    // are transient and must not hide the window (e.g. UWP apps like Settings stay cloaked
    // while suspended, but ours only last for the animation).
    if !window.is_window()
        || !window.is_visible()
        || (window.is_cloaked() && !window.is_cloaked_by_seelen())
    {
        return false;
    }

    // Removed from the taskbar by the app itself via `ITaskbarList::DeleteTab`
    if window.is_taskbar_tab_deleted() {
        return false;
    }

    // this class is used for edge tabs to be shown as independent windows on alt + tab
    // this only applies when the new tab is created it is bound to explorer.exe for some reason
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
