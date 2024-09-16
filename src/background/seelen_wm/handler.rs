use tauri::{Webview, Wry};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOOWNERZORDER, SWP_NOSENDCHANGING,
        SWP_NOZORDER,
    },
};

use crate::{seelen::SEELEN, trace_lock, windows_api::WindowsApi};
use seelen_core::rect::Rect;

#[tauri::command(async)]
pub fn set_window_position(hwnd: isize, rect: Rect) -> Result<(), String> {
    let hwnd = HWND(hwnd as _);

    if !WindowsApi::is_window(hwnd) || WindowsApi::is_iconic(hwnd) {
        return Ok(());
    }

    WindowsApi::unmaximize_window(hwnd)?;
    let shadow = WindowsApi::shadow_rect(hwnd)?;
    WindowsApi::set_position(
        hwnd,
        None,
        &RECT {
            top: rect.top + shadow.top,
            left: rect.left + shadow.left,
            right: rect.right + shadow.right,
            bottom: rect.bottom + shadow.bottom,
        },
        SWP_NOACTIVATE
            | SWP_NOCOPYBITS
            | SWP_NOZORDER
            | SWP_NOOWNERZORDER
            | SWP_ASYNCWINDOWPOS
            | SWP_NOSENDCHANGING,
    )?;
    Ok(())
}

#[tauri::command(async)]
pub fn bounce_handle(webview: Webview<Wry>, hwnd: isize) {
    let monitor_id = webview.label().split("/").last().expect("No monitor ID");
    let monitor_id = monitor_id.parse::<isize>().expect("Invalid monitor ID");

    if let Some(monitor) = trace_lock!(SEELEN).monitor_by_id_mut(monitor_id) {
        if let Some(wm) = monitor.wm_mut() {
            wm.bounce_handle(HWND(hwnd as _));
        }
    }
}

#[tauri::command(async)]
pub fn request_focus(hwnd: isize) -> Result<(), String> {
    let hwnd = HWND(hwnd as _);
    log::trace!(
        "Requesting focus on {:?} - {} , {:?}",
        hwnd,
        WindowsApi::get_window_text(hwnd),
        WindowsApi::exe(hwnd)?,
    );

    if !WindowsApi::is_window(hwnd) {
        return Ok(());
    }

    WindowsApi::force_set_foreground(hwnd)?;
    Ok(())
}
