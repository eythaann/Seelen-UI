use std::path::PathBuf;

use seelen_core::{handlers::SeelenEvent, state::WegItemData};
use tauri_plugin_shell::ShellExt;

use crate::{
    app::{emit_to_webviews, get_app_handle},
    error::Result,
    state::application::WEG_ITEMS_MANAGER,
    windows_api::{window::Window, WindowsApi},
};
use windows::Win32::UI::WindowsAndMessaging::{SW_MINIMIZE, WM_CLOSE};

#[tauri::command(async)]
pub fn weg_close_app(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    WindowsApi::post_message(window.hwnd(), WM_CLOSE, 0, 0)?;
    Ok(())
}

#[tauri::command(async)]
pub fn weg_kill_app(hwnd: isize) -> Result<()> {
    let window = Window::from(hwnd);
    get_app_handle()
        .shell()
        .command("taskkill.exe")
        .args(["/F", "/PID", &window.process().id().to_string()])
        .spawn()?;
    Ok(())
}

#[tauri::command(async)]
pub fn weg_toggle_window_state(hwnd: isize, was_focused: bool) -> Result<()> {
    let window = Window::from(hwnd);
    // was_focused is intented to know if the window was focused before click on the dock item
    // on click the items makes the dock being focused.
    if was_focused {
        window.show_window_async(SW_MINIMIZE)?;
    } else {
        window.unminimize()?;
        window.focus()?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn weg_pin_item(path: PathBuf) -> Result<()> {
    if !path.exists() || path.is_dir() {
        return Err("Invalid path".into());
    }

    let umid = WindowsApi::get_file_umid(&path).ok();
    let display_name = WindowsApi::get_executable_display_name(&path).unwrap_or_else(|_| {
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    });

    let item = WegItemData {
        id: uuid::Uuid::new_v4(),
        display_name,
        umid,
        path,
        pinned: true,
        prevent_pinning: false,
        relaunch: None,
    };

    emit_to_webviews(SeelenEvent::WegAddItem, &item);
    Ok(())
}

#[tauri::command(async)]
pub fn weg_import_pinned_taskbar_items() -> Result<usize> {
    let count = WEG_ITEMS_MANAGER.import_from_windows_taskbar()?;

    // Emit event to notify all webviews about the updated items
    if count > 0 {
        let items = WEG_ITEMS_MANAGER.get();
        emit_to_webviews(SeelenEvent::WegItemsChanged, items);
    }

    Ok(count)
}
