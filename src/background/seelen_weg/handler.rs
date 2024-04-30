use std::path::PathBuf;

use image::ImageFormat;
use serde::Deserialize;
use tauri::{command, Manager};
use tauri_plugin_shell::ShellExt;

use crate::{seelen::SEELEN, windows_api::WindowsApi};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, ShowWindow, SW_MINIMIZE, SW_RESTORE, WM_CLOSE},
};

use super::SeelenWeg;

#[command]
pub fn enum_opened_apps() {
    std::thread::spawn(|| {
        if let Some(weg) = SEELEN.lock().weg() {
            weg.update_ui();
        }
    });
}

#[derive(Deserialize)]
pub struct Args {
    hwnd: isize,
    process_hwnd: isize,
}
#[command]
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
                let mut output_path = PathBuf::from(temp_dir.clone());
                output_path.push(format!("{}.png", hwnd.0));
                image
                    .save_with_format(&output_path, ImageFormat::Png)
                    .expect("could not save image");
                SEELEN
                    .lock()
                    .handle()
                    .emit(format!("weg-preview-update-{}", hwnd.0).as_str(), ())
                    .expect("could not emit event");
            }
        }
    });
    Ok(())
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
pub fn weg_toggle_window_state(hwnd: isize, exe_path: String) -> Result<(), String> {
    let hwnd = HWND(hwnd);

    if WindowsApi::is_window(hwnd) {
        if WindowsApi::is_cloaked(hwnd)? {
            WindowsApi::force_set_foreground(hwnd)?;
            return Ok(());
        }

        if WindowsApi::is_iconic(hwnd) {
            unsafe { ShowWindow(hwnd, SW_RESTORE) };
        } else {
            unsafe { ShowWindow(hwnd, SW_MINIMIZE) };
        }
    } else {
        SEELEN
            .lock()
            .handle()
            .shell()
            .command("explorer")
            .arg(&exe_path)
            .spawn()
            .expect("Could not spawn explorer on Opening App Action");
    }

    Ok(())
}
