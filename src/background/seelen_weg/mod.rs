pub mod cli;
pub mod handler;
pub mod hook;
pub mod icon_extractor;
pub mod instance;
pub mod weg_items_impl;

pub use instance::SeelenWeg;

use std::{collections::HashMap, thread::JoinHandle};

use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{handlers::SeelenEvent, state::AppExtraFlag};
use tauri::Emitter;
use weg_items_impl::WEG_ITEMS_IMPL;
use win_screenshot::capture::capture_window;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL},
};

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    state::application::FULL_STATE,
    trace_lock,
    utils::sleep_millis,
    windows_api::{window::Window, AppBarData, AppBarDataState, WindowEnumerator, WindowsApi},
};

lazy_static! {
    static ref TITLE_BLACK_LIST: Vec<&'static str> = Vec::from([
        "",
        "Task Switching",
        "DesktopWindowXamlSource",
        "Program Manager",
    ]);
}

impl SeelenWeg {
    pub fn contains_app(window: &Window) -> bool {
        trace_lock!(WEG_ITEMS_IMPL).contains(window)
    }

    pub fn update_app(_window: &Window) {
        // TODO implement
    }

    pub fn enumerate_all_windows() -> Result<()> {
        WindowEnumerator::new().for_each(|hwnd| {
            let window = Window::from(hwnd);
            if Self::should_be_added(&window) {
                log_error!(Self::add(&window));
            }
        })
    }

    pub fn add(window: &Window) -> Result<()> {
        let mut items = trace_lock!(WEG_ITEMS_IMPL);
        items.add(window)?;
        get_app_handle()
            .emit(SeelenEvent::WegInstanceChanged, items.get())
            .expect("Failed to emit");
        Ok(())
    }

    pub fn remove_hwnd(window: &Window) {
        let mut items = trace_lock!(WEG_ITEMS_IMPL);
        items.remove(window);
        get_app_handle()
            .emit(SeelenEvent::WegInstanceChanged, items.get())
            .expect("Failed to emit");
    }

    pub fn should_be_added(window: &Window) -> bool {
        if !window.is_real_window() {
            return false;
        }

        if let Some(config) = FULL_STATE.load().get_app_config_by_window(window.hwnd()) {
            if config.options.contains(&AppExtraFlag::Hidden) {
                log::trace!("Skipping by config: {:?}", window);
                return false;
            }
        }

        !TITLE_BLACK_LIST.contains(&window.title().as_str())
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

lazy_static! {
    pub static ref TASKBAR_STATE_ON_INIT: Mutex<HashMap<isize, AppBarDataState>> =
        Mutex::new(HashMap::new());
    pub static ref TASKBAR_CLASS: Vec<&'static str> =
        Vec::from(["Shell_TrayWnd", "Shell_SecondaryTrayWnd",]);
}

pub fn get_taskbars_handles() -> Result<Vec<HWND>> {
    let mut founds = Vec::new();
    WindowEnumerator::new().for_each(|hwnd| {
        let class = WindowsApi::get_class(hwnd).unwrap_or_default();
        if TASKBAR_CLASS.contains(&class.as_str()) {
            founds.push(hwnd);
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
                        trace_lock!(TASKBAR_STATE_ON_INIT)
                            .insert(handle.0 as isize, app_bar.state());
                        app_bar.set_state(AppBarDataState::AutoHide);
                        let _ = WindowsApi::show_window(*handle, SW_HIDE);
                    }
                    attempts += 1;
                    sleep_millis(50);
                }
            }
            Err(err) => log::error!("Failed to get taskbars handles: {:?}", err),
        })
    }

    pub fn restore_taskbar() -> Result<()> {
        for hwnd in get_taskbars_handles()? {
            AppBarData::from_handle(hwnd).set_state(
                *trace_lock!(TASKBAR_STATE_ON_INIT)
                    .get(&(hwnd.0 as isize))
                    .unwrap_or(&AppBarDataState::AlwaysOnTop),
            );
            WindowsApi::show_window(hwnd, SW_SHOWNORMAL)?;
        }
        Ok(())
    }
}
