use windows::Win32::{
    Foundation::RECT,
    UI::WindowsAndMessaging::{
        SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING, SW_NORMAL,
    },
};

use crate::{
    error_handler::Result,
    windows_api::{window::Window, WindowsApi},
};
use seelen_core::rect::Rect;

#[tauri::command(async)]
pub fn set_window_position(hwnd: isize, rect: Rect) -> Result<()> {
    let window = Window::from(hwnd);

    if !window.is_window() || window.is_minimized() {
        return Ok(());
    }

    window.show_window_async(SW_NORMAL)?;

    let shadow = WindowsApi::shadow_rect(window.hwnd())?;
    let rect = RECT {
        top: rect.top + shadow.top,
        left: rect.left + shadow.left,
        right: rect.right + shadow.right,
        bottom: rect.bottom + shadow.bottom,
    };

    // WindowsApi::move_window(hwnd, &rect)?;
    window.set_position(
        &rect,
        SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_ASYNCWINDOWPOS | SWP_NOSENDCHANGING,
    )?;
    Ok(())
}

#[tauri::command(async)]
pub fn request_focus(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    if !window.is_window() {
        return Ok(());
    }
    window.focus()?;
    Ok(())
}
