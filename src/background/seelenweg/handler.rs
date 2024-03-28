use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use tauri::command;

use crate::{seelen::SEELEN, windows_api::WindowsApi};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::{
                PostMessageW, ShowWindow, SW_MINIMIZE, SW_RESTORE, SW_SHOWNORMAL, WM_CLOSE,
            },
        },
    },
};

#[command]
pub fn weg_request_apps() {
    SEELEN.lock().weg().update_ui();
}

#[command]
pub fn weg_close_app(hwnd: isize) -> Result<(), String> {
    let hwnd = HWND(hwnd);
    unsafe {
        match PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)) {
            Ok(()) => Ok(()),
            Err(_) => Err("could not close window".to_owned()),
        }
    }
}

#[command]
pub fn weg_toggle_window_state(hwnd: isize, exe_path: String) {
    let hwnd = HWND(hwnd);

    if WindowsApi::is_window(hwnd) {
        if WindowsApi::is_iconic(hwnd) {
            unsafe { ShowWindow(hwnd, SW_RESTORE) };
        } else {
            unsafe { ShowWindow(hwnd, SW_MINIMIZE) };
        }
    } else {
        let wide_file_path: Vec<u16> = OsStr::new(&exe_path)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();

        let operation: Vec<u16> = OsStr::new("open")
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect();

        unsafe {
            ShellExecuteW(
                HWND(0),
                PCWSTR(operation.as_ptr()),
                PCWSTR(wide_file_path.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            );
        }
    }
}
