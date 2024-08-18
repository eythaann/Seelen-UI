use image::ImageFormat;
use serde::Deserialize;
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;

use crate::{error_handler::Result, seelen::get_app_handle, windows_api::WindowsApi};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, SW_MINIMIZE, SW_RESTORE, WM_CLOSE},
};

use super::SeelenWeg;

#[derive(Deserialize)]
pub struct Args {
    hwnd: isize,
    process_hwnd: isize,
}
#[tauri::command(async)]
pub fn weg_request_update_previews(hwnds: Vec<Args>) -> Result<(), String> {
    std::thread::spawn(move || {
        for app in hwnds {
            if WindowsApi::is_iconic(HWND(app.hwnd)) {
                continue;
            }

            let temp_dir = std::env::temp_dir();
            let hwnd = HWND(app.process_hwnd);
            let image = SeelenWeg::capture_window(hwnd);
            if let Some(image) = image {
                let mut output_path = temp_dir.clone();
                output_path.push(format!("{}.png", hwnd.0));
                image
                    .save_with_format(&output_path, ImageFormat::Png)
                    .expect("could not save image");
                get_app_handle()
                    .emit(format!("weg-preview-update-{}", hwnd.0).as_str(), ())
                    .expect("could not emit event");
            }
        }
    });
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
pub fn weg_toggle_window_state(hwnd: isize, exe_path: String) -> Result<()> {
    let hwnd = HWND(hwnd);

    if WindowsApi::is_window(hwnd) {
        if WindowsApi::is_cloaked(hwnd)? {
            WindowsApi::force_set_foreground(hwnd)?;
            return Ok(());
        }

        if WindowsApi::is_iconic(hwnd) {
            WindowsApi::show_window(hwnd, SW_RESTORE)?;
        } else {
            WindowsApi::show_window(hwnd, SW_MINIMIZE)?;
        }
    } else {
        get_app_handle()
            .shell()
            .command("explorer")
            .arg(&exe_path)
            .spawn()
            .expect("Could not spawn explorer on Opening App Action");
    }

    Ok(())
}
