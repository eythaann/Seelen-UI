use serde::Deserialize;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        SetWindowPos, HWND_BOTTOM, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOCOPYBITS,
        SWP_NOOWNERZORDER, SWP_NOSENDCHANGING, SWP_NOZORDER,
    },
};

use crate::{error_handler::log_if_error, seelen::SEELEN, windows_api::WindowsApi};

#[derive(Deserialize, Debug)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[tauri::command]
pub fn set_window_position(hwnd: isize, rect: Rect) {
    let hwnd = HWND(hwnd);

    if !WindowsApi::is_window(hwnd) {
        return;
    }

    let shadow = WindowsApi::shadow_rect(hwnd).unwrap();
    let result = unsafe {
        SetWindowPos(
            hwnd,
            HWND(0),
            rect.left + shadow.left,
            rect.top + shadow.top,
            rect.right + shadow.right + shadow.left.abs(),
            rect.bottom + shadow.bottom + shadow.top.abs(),
            SWP_NOACTIVATE
                | SWP_NOCOPYBITS
                | SWP_NOZORDER
                | SWP_NOOWNERZORDER
                | SWP_ASYNCWINDOWPOS
                | SWP_NOSENDCHANGING,
        )
    };

    log_if_error(result);
}

#[tauri::command]
pub fn remove_hwnd(hwnd: isize) {
    if let Some(wm) = SEELEN.lock().wm_mut() {
        wm.remove_hwnd_no_emit(HWND(hwnd))
    }
}