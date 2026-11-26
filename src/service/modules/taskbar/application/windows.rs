use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL},
};

use crate::{
    error::Result,
    modules::taskbar::domain::TaskbarSystem,
    windows_api::{
        WindowsApi,
        app_bar::{AppBarData, AppBarDataState},
        iterator::WindowEnumerator,
    },
};

const PRIMARY_TASKBAR_CLASS: &str = "Shell_TrayWnd";
const SECONDARY_TASKBAR_CLASS: &str = "Shell_SecondaryTrayWnd";

pub struct WindowsTaskbar;

impl TaskbarSystem for WindowsTaskbar {
    fn create() -> Self {
        Self
    }

    fn hide<F>(&self, may_move_focus: bool, still_hidden: F) -> Result<bool>
    where
        F: Fn() -> bool,
    {
        // Enumerated on every check: taskbars can be created (explorer restart, new monitor)
        // or destroyed while a pass is running.
        let handles = get_taskbars_handles()?;
        if handles.is_empty() {
            return Ok(false);
        }

        // ABM_SETSTATE makes explorer re-run its auto-hide logic and show the taskbar again,
        // which fires a new show event and so a new hide request. On Windows 10 21H2 that code
        // path can also crash explorer, so only send it when the state really has to change.
        // The auto-hide state is shared by all the taskbars, it belongs to the primary one.
        let primary = handles
            .iter()
            .find(|hwnd| WindowsApi::get_class(**hwnd) == PRIMARY_TASKBAR_CLASS)
            .unwrap_or(&handles[0]);
        let app_bar = AppBarData::from_handle(*primary);
        if !app_bar.is_auto_hide() {
            app_bar.set_state(AppBarDataState::AutoHide);
        }

        // Explorer keeps a focused taskbar shown, so move the focus to the desktop first.
        // Only once per pass, as it can fall back to simulating an alt tap.
        let mut focus_moved = false;
        let foreground = WindowsApi::get_foreground_window();
        if may_move_focus && handles.contains(&foreground) {
            focus_moved = true;
            let desktop = WindowsApi::get_shell_window();
            if !desktop.is_invalid() {
                log::debug!("Native taskbar is focused, moving the focus to the desktop");
                if let Err(err) = WindowsApi::set_foreground(desktop.0 as isize) {
                    log::warn!("Failed to move the focus out of the native taskbar: {err:?}");
                }
            }
        }

        // moving the focus can take a while, don't hide a taskbar restored meanwhile
        if !still_hidden() {
            return Ok(focus_moved);
        }
        for hwnd in &handles {
            if WindowsApi::is_window_visible(*hwnd) {
                log::debug!("Hiding visible native taskbar {:?}", hwnd.0);
                let _ = WindowsApi::show_window_async(hwnd.0 as isize, SW_HIDE.0);
            }
        }
        Ok(focus_moved)
    }

    fn restore(&self) -> Result<()> {
        for hwnd in get_taskbars_handles()? {
            AppBarData::from_handle(hwnd).set_state(AppBarDataState::AlwaysOnTop);
            WindowsApi::show_window_async(hwnd.0 as isize, SW_SHOWNORMAL.0)?;
        }
        Ok(())
    }
}

fn get_taskbars_handles() -> Result<Vec<HWND>> {
    let mut founds = Vec::new();
    WindowEnumerator::new().for_each(|hwnd| {
        let class = WindowsApi::get_class(hwnd);
        if (class == PRIMARY_TASKBAR_CLASS || class == SECONDARY_TASKBAR_CLASS)
            && WindowsApi::get_title(hwnd).is_empty()
        {
            founds.push(hwnd);
        }
    })?;
    Ok(founds)
}
