use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS;

use crate::{error::Result, windows_api::WindowsApi};

use super::SeelenWall;

#[tauri::command(async)]
pub fn set_as_wallpaper(webview: tauri::WebviewWindow) -> Result<()> {
    let hwnd = HWND(webview.hwnd()?.0);
    let rect = WindowsApi::virtual_screen_rect()?;

    match SeelenWall::try_set_under_desktop_items(hwnd) {
        Ok(_) => {
            let relative_rect = RECT {
                left: 0,
                top: 0,
                right: rect.right - rect.left,
                bottom: rect.bottom - rect.top,
            };
            webview.set_always_on_bottom(false)?;
            WindowsApi::move_window(hwnd, &relative_rect)?;
            WindowsApi::set_position(hwnd, None, &relative_rect, SWP_ASYNCWINDOWPOS)?;
        }
        Err(e) => {
            log::warn!(
                "Failed to attach to desktop hierarchy: {}, using absolute positioning",
                e
            );
            webview.set_always_on_bottom(true)?;
            WindowsApi::move_window(hwnd, &rect)?;
            WindowsApi::set_position(hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
        }
    }

    SeelenWall::refresh_desktop()?;
    Ok(())
}
