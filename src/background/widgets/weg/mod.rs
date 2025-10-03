pub mod cli;
pub mod handler;
pub mod hook;
pub mod instance;
pub mod weg_items_impl;

pub use instance::SeelenWeg;

use std::thread::JoinHandle;

use image::{DynamicImage, RgbaImage};
use weg_items_impl::SEELEN_WEG_STATE;
use win_screenshot::capture::capture_window;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL},
};

use crate::{
    error::Result,
    state::application::FULL_STATE,
    trace_lock,
    utils::sleep_millis,
    windows_api::{window::Window, AppBarData, AppBarDataState, WindowEnumerator, WindowsApi},
};

impl SeelenWeg {
    pub fn contains_app(window: &Window) -> bool {
        trace_lock!(SEELEN_WEG_STATE).contains(window)
    }

    pub fn foregrounded_app(window: &Window) -> Result<()> {
        let mut weg = trace_lock!(SEELEN_WEG_STATE);
        weg.update_window_activation(window);
        weg.emit_to_webview()?;
        Ok(())
    }

    pub fn update_app(window: &Window) -> Result<()> {
        let mut weg = trace_lock!(SEELEN_WEG_STATE);
        weg.update_window_info(window);
        weg.emit_to_webview()?;
        Ok(())
    }

    pub fn capture_window(hwnd: HWND) -> Option<DynamicImage> {
        capture_window(hwnd.0 as isize).ok().map(|buf| {
            let image = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap_or_default();
            DynamicImage::ImageRgba8(image)
        })
    }
}

// ====================
// TASKBAR HIDDEN LOGIC
// ====================

pub static TASKBAR_CLASS: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

pub fn get_taskbars_handles() -> Result<Vec<HWND>> {
    let mut founds = Vec::new();
    WindowEnumerator::new().for_each(|w| {
        if TASKBAR_CLASS.contains(&w.class().as_str()) {
            founds.push(w.hwnd());
        }
    })?;
    Ok(founds)
}

impl SeelenWeg {
    pub fn hide_taskbar() -> JoinHandle<()> {
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

    pub fn restore_taskbar() -> Result<()> {
        for hwnd in get_taskbars_handles()? {
            AppBarData::from_handle(hwnd).set_state(AppBarDataState::AlwaysOnTop);
            WindowsApi::show_window_async(hwnd, SW_SHOWNORMAL)?;
        }
        Ok(())
    }
}
