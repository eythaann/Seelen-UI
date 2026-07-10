pub mod cli;
pub mod handler;
pub mod hook;
pub mod instance;

pub use instance::SeelenWeg;

use std::thread::JoinHandle;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL},
};

use crate::{
    error::Result,
    state::application::FULL_STATE,
    utils::sleep_millis,
    windows_api::{AppBarData, AppBarDataState, WindowEnumerator, WindowsApi},
};

// ====================
// TASKBAR HIDDEN LOGIC
// ====================

pub static TASKBAR_CLASS: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

pub fn get_taskbars_handles() -> Result<Vec<HWND>> {
    let mut founds = Vec::new();
    WindowEnumerator::new().for_each(|w| {
        if TASKBAR_CLASS.contains(&w.class().as_str()) && w.title().is_empty() {
            founds.push(w.hwnd());
        }
    })?;
    Ok(founds)
}

impl SeelenWeg {
    pub fn hide_native_taskbar() -> JoinHandle<()> {
        std::thread::spawn(move || match get_taskbars_handles() {
            Ok(handles) => {
                let mut attempts = 0;
                while attempts < 10 && FULL_STATE.load().is_weg_enabled() {
                    for handle in &handles {
                        let app_bar = AppBarData::from_handle(*handle);
                        app_bar.set_state(AppBarDataState::AutoHide);
                        let _ = WindowsApi::show_window_async(*handle, SW_HIDE);
                    }
                    attempts += 1;
                    sleep_millis(50);
                }
            }
            Err(err) => log::error!("Failed to get taskbars handles: {err:?}"),
        })
    }

    pub fn restore_native_taskbar() -> Result<()> {
        for hwnd in get_taskbars_handles()? {
            AppBarData::from_handle(hwnd).set_state(AppBarDataState::AlwaysOnTop);
            WindowsApi::show_window_async(hwnd, SW_SHOWNORMAL)?;
        }
        Ok(())
    }
}
