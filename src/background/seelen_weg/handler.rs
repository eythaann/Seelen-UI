use std::{ffi::OsStr, path::PathBuf, sync::atomic::Ordering};

use image::ImageFormat;
use seelen_core::state::{PinnedWegItemData, WegItem};
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;

use crate::{
    error_handler::Result, hook::LAST_ACTIVE_NOT_SEELEN, seelen::get_app_handle,
    state::application::FULL_STATE, trace_lock, windows_api::WindowsApi,
};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SW_MINIMIZE, SW_RESTORE, WM_CLOSE},
};

use super::SeelenWeg;

#[tauri::command(async)]
pub fn weg_request_update_previews(handles: Vec<isize>) -> Result<()> {
    let temp_dir = std::env::temp_dir();

    for addr in handles {
        let hwnd: HWND = HWND(addr as _);

        if hwnd.is_invalid() || !WindowsApi::is_window_visible(hwnd) {
            SeelenWeg::remove_hwnd(hwnd);
            continue;
        }

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
    let hwnd = HWND(hwnd as _);
    if !WindowsApi::is_window_visible(hwnd) {
        SeelenWeg::remove_hwnd(hwnd);
    } else {
        WindowsApi::post_message(hwnd, WM_CLOSE, 0, 0)?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn weg_kill_app(hwnd: isize) -> Result<()> {
    let hwnd = HWND(hwnd as _);
    if !WindowsApi::is_window_visible(hwnd) {
        SeelenWeg::remove_hwnd(hwnd);
    } else {
        let (pid, _) = WindowsApi::window_thread_process_id(hwnd);
        get_app_handle()
            .shell()
            .command("taskkill.exe")
            .args(["/F", "/PID", &pid.to_string()])
            .spawn()?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn weg_toggle_window_state(hwnd: isize) -> Result<()> {
    let hwnd = HWND(hwnd as _);

    if hwnd.is_invalid() || !WindowsApi::is_window_visible(hwnd) {
        SeelenWeg::remove_hwnd(hwnd);
        return Ok(());
    }

    if WindowsApi::is_iconic(hwnd) {
        WindowsApi::show_window_async(hwnd, SW_RESTORE)?;
        return Ok(());
    }

    let last_active = LAST_ACTIVE_NOT_SEELEN.load(Ordering::Acquire);
    if last_active == hwnd.0 as isize {
        WindowsApi::show_window_async(hwnd, SW_MINIMIZE)?;
    } else {
        WindowsApi::set_foreground(hwnd)?;
    }

    Ok(())
}

#[tauri::command(async)]
pub fn weg_pin_item(path: PathBuf) -> Result<()> {
    // todo add support to UWP for seelen rofi
    let mut data = PinnedWegItemData {
        path: path.clone(),
        is_dir: path.is_dir(),
        execution_command: path.to_string_lossy().to_string(),
    };

    if path.extension() == Some(OsStr::new("lnk")) {
        let (program, _arguments) = WindowsApi::resolve_lnk_target(&path)?;
        data.is_dir = program.is_dir();
        data.execution_command = program.to_string_lossy().to_string();
    }

    let state = FULL_STATE.load();
    let mut weg_items = trace_lock!(state.weg_items);
    weg_items.center.insert(0, WegItem::Pinned(data));
    weg_items.sanitize();
    state.emit_weg_items(&weg_items)?;
    state.save_weg_items(&weg_items)?;
    Ok(())
}
