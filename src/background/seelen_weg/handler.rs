use std::{ffi::OsStr, path::PathBuf, sync::atomic::Ordering};

use image::ImageFormat;
use seelen_core::state::{PinnedWegItemData, WegItem, WegItems};
use tauri::Emitter;
use tauri_plugin_shell::ShellExt;

use crate::{
    error_handler::Result,
    hook::LAST_ACTIVE_NOT_SEELEN,
    seelen::get_app_handle,
    seelen_weg::weg_items_impl::WEG_ITEMS_IMPL,
    state::application::FULL_STATE,
    trace_lock,
    windows_api::{window::Window, WindowsApi},
};
use windows::Win32::UI::WindowsAndMessaging::{SW_MINIMIZE, SW_RESTORE, WM_CLOSE};

use super::SeelenWeg;

#[tauri::command(async)]
pub fn weg_get_items_for_widget() -> WegItems {
    trace_lock!(WEG_ITEMS_IMPL).get()
}

#[tauri::command(async)]
pub fn weg_request_update_previews(handles: Vec<isize>) -> Result<()> {
    let temp_dir = std::env::temp_dir();

    for addr in handles {
        let window = Window::from(addr);

        if !window.is_visible() {
            SeelenWeg::remove_hwnd(&window);
            continue;
        }

        if window.is_minimized() {
            continue;
        }

        let image = SeelenWeg::capture_window(window.hwnd());
        if let Some(image) = image {
            let rect = WindowsApi::get_inner_window_rect(window.hwnd())?;
            let shadow = WindowsApi::shadow_rect(window.hwnd())?;
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
    let window = Window::from(hwnd);
    if !window.is_visible() {
        SeelenWeg::remove_hwnd(&window);
    } else {
        WindowsApi::post_message(window.hwnd(), WM_CLOSE, 0, 0)?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn weg_kill_app(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    if !window.is_visible() {
        SeelenWeg::remove_hwnd(&window);
    } else {
        get_app_handle()
            .shell()
            .command("taskkill.exe")
            .args(["/F", "/PID", &window.process().id().to_string()])
            .spawn()?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn weg_toggle_window_state(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    if !window.is_visible() {
        SeelenWeg::remove_hwnd(&window);
        return Ok(());
    }

    if window.is_minimized() {
        WindowsApi::show_window_async(window.hwnd(), SW_RESTORE)?;
        return Ok(());
    }

    let last_active = LAST_ACTIVE_NOT_SEELEN.load(Ordering::Acquire);
    if last_active == window.address() {
        WindowsApi::show_window_async(window.hwnd(), SW_MINIMIZE)?;
    } else {
        WindowsApi::set_foreground(window.hwnd())?;
    }

    Ok(())
}

#[tauri::command(async)]
pub fn weg_pin_item(path: PathBuf) -> Result<()> {
    let display_name = if let Some(name) = path.file_name() {
        name.to_string_lossy().to_string()
    } else {
        "Unknown".to_string()
    };

    // todo add support to UWP for seelen rofi
    let mut data = PinnedWegItemData {
        id: uuid::Uuid::new_v4().to_string(),
        umid: WindowsApi::get_file_umid(&path).ok(),
        display_name,
        path: path.clone(),
        is_dir: path.is_dir(),
        relaunch_command: path.to_string_lossy().to_string(),
        windows: vec![],
    };

    if path.extension() == Some(OsStr::new("lnk")) {
        let (program, _arguments) = WindowsApi::resolve_lnk_target(&path)?;
        data.is_dir = program.is_dir();
        data.relaunch_command = program.to_string_lossy().to_string();
    }

    FULL_STATE.rcu(move |state| {
        let mut state = state.cloned();
        state
            .weg_items
            .center
            .insert(0, WegItem::Pinned(data.clone()));
        state.weg_items.sanitize();
        state
    });
    FULL_STATE.load().write_weg_items()?;
    Ok(())
}
