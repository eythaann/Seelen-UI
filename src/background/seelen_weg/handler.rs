use std::{path::PathBuf, sync::atomic::Ordering};

use image::ImageFormat;
use seelen_core::state::WegItem;
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;

use crate::{
    error_handler::Result, hook::LAST_ACTIVE_NOT_SEELEN, modules::uwp::UWP_MANAGER,
    seelen::get_app_handle, state::application::FULL_STATE, trace_lock, windows_api::WindowsApi,
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, SW_MINIMIZE, SW_RESTORE, WM_CLOSE},
};

use super::SeelenWeg;

#[tauri::command(async)]
pub fn weg_request_update_previews(handles: Vec<isize>) -> Result<()> {
    let temp_dir = std::env::temp_dir();

    for addr in handles {
        let hwnd: HWND = HWND(addr as _);

        if WindowsApi::is_iconic(hwnd) {
            continue;
        }

        let image = SeelenWeg::capture_window(hwnd);
        if let Some(image) = image {
            let rect = WindowsApi::get_inner_window_rect(hwnd)?;
            let shadow = WindowsApi::shadow_rect(hwnd)?;
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let image = image.crop_imm(
                shadow.left.unsigned_abs(),
                shadow.top.unsigned_abs(),
                width as u32,
                height as u32,
            );

            image.save_with_format(temp_dir.join(format!("{}.png", addr)), ImageFormat::Png)?;
            get_app_handle().emit(format!("weg-preview-update-{}", addr).as_str(), ())?;
        }
    }
    Ok(())
}

#[tauri::command(async)]
pub fn weg_close_app(hwnd: isize) -> Result<()> {
    unsafe { PostMessageW(HWND(hwnd as _), WM_CLOSE, WPARAM(0), LPARAM(0))? };
    Ok(())
}

#[tauri::command(async)]
pub fn weg_toggle_window_state(hwnd: isize, exe_path: String) -> Result<()> {
    let hwnd = HWND(hwnd as _);

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
        WindowsApi::show_window(hwnd, SW_RESTORE)?;
        return Ok(());
    }

    if LAST_ACTIVE_NOT_SEELEN.load(Ordering::Acquire) == hwnd.0 as isize {
        WindowsApi::show_window(hwnd, SW_MINIMIZE)?;
    } else {
        WindowsApi::async_force_set_foreground(hwnd)
    }

    Ok(())
}

#[tauri::command(async)]
pub fn weg_pin_item(path: PathBuf) -> Result<()> {
    let mut state = FULL_STATE.load().cloned();

    let item = if path.ends_with(".exe") {
        let mut execution_path = None;
        if let Some(package) = trace_lock!(UWP_MANAGER, 10).get_from_path(&path) {
            if let Some(app) = path.file_name() {
                execution_path = package.get_shell_path(app.to_string_lossy().as_ref());
            }
        }
        WegItem::PinnedApp {
            exe: path.clone(),
            execution_path: execution_path.unwrap_or(path.to_string_lossy().to_string()),
        }
    } else {
        WegItem::Pinned {
            is_dir: path.is_dir(),
            path,
        }
    };

    state.weg_items.center.insert(0, item);
    state.save_weg_items()?;
    Ok(())
}
