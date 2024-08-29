use image::ImageFormat;
use tauri::{Emitter, WebviewWindow};
use tauri_plugin_shell::ShellExt;

use crate::{error_handler::Result, seelen::get_app_handle, windows_api::WindowsApi};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, SW_MINIMIZE, SW_RESTORE, SW_SHOWNORMAL, WM_CLOSE},
};

use super::SeelenWeg;

#[tauri::command(async)]
pub fn weg_request_update_previews(handles: Vec<isize>) -> Result<()> {
    let temp_dir = std::env::temp_dir();

    for hwnd in handles {
        let hwnd: HWND = HWND(hwnd);

        if WindowsApi::is_iconic(hwnd) {
            continue;
        }

        let image = SeelenWeg::capture_window(hwnd);
        if let Some(image) = image {
            let rect = WindowsApi::get_window_rect_without_margins(hwnd);
            let shadow = WindowsApi::shadow_rect(hwnd)?;
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let image = image.crop_imm(
                shadow.left.unsigned_abs(),
                shadow.top.unsigned_abs(),
                width as u32,
                height as u32,
            );

            image.save_with_format(temp_dir.join(format!("{}.png", hwnd.0)), ImageFormat::Png)?;
            get_app_handle().emit(format!("weg-preview-update-{}", hwnd.0).as_str(), ())?;
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn weg_close_app(hwnd: isize) -> Result<(), String> {
    let hwnd = HWND(hwnd);
    unsafe {
        match PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0)) {
            Ok(()) => Ok(()),
            Err(_) => Err("could not close window".to_owned()),
        }
    }
}

#[tauri::command(async)]
pub fn weg_toggle_window_state(window: WebviewWindow, hwnd: isize, exe_path: String) -> Result<()> {
    let hwnd = HWND(hwnd);

    // If the window is not open, open it
    if !WindowsApi::is_window(hwnd) {
        get_app_handle()
            .shell()
            .command("explorer")
            .arg(&exe_path)
            .spawn()?;
        return Ok(());
    }

    if WindowsApi::is_iconic(hwnd) {
        println!("iconic");

        WindowsApi::show_window(hwnd, SW_SHOWNORMAL)?;
        WindowsApi::show_window(hwnd, SW_RESTORE)?;
        return Ok(());
    }

    let foreground = WindowsApi::get_foreground_window();
    if foreground == hwnd || foreground == window.hwnd()? {
        WindowsApi::show_window(hwnd, SW_MINIMIZE)?;
    } else {
        WindowsApi::force_set_foreground(hwnd)?;
    }

    Ok(())
}
