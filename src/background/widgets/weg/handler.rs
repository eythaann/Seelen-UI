use std::{ffi::OsStr, path::PathBuf};

use seelen_core::{
    state::{PinnedWegItemData, RelaunchArguments, WegItem, WegItemSubtype, WegItems},
    system_state::MonitorId,
};
use tauri_plugin_shell::ShellExt;

use crate::{
    app::get_app_handle,
    error::Result,
    state::application::FULL_STATE,
    trace_lock,
    widgets::weg::weg_items_impl::SEELEN_WEG_STATE,
    windows_api::{window::Window, WindowsApi},
};
use windows::Win32::UI::WindowsAndMessaging::{SW_SHOWMINNOACTIVE, WM_CLOSE};

#[tauri::command(async)]
pub fn state_get_weg_items(monitor_id: Option<MonitorId>) -> WegItems {
    let guard = trace_lock!(SEELEN_WEG_STATE);
    if let Some(id) = monitor_id {
        return guard
            .get_filtered_by_monitor()
            .unwrap_or_default()
            .get(&id)
            .cloned()
            .unwrap_or_else(|| guard.items.clone());
    }
    guard.items.clone()
}

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
        // Got to prevent the activation, because the click initialed as Seelen in focus, and the
        // activation here will make this assigned to an app, which is not properly focused, just activated.
        window.show_window_async(SW_SHOWMINNOACTIVE)?;
    } else {
        window.focus()?;
    }
    Ok(())
}

#[allow(deprecated)]
#[tauri::command(async)]
pub fn weg_pin_item(path: PathBuf) -> Result<()> {
    let display_name = if let Some(name) = path.file_name() {
        name.to_string_lossy().to_string()
    } else {
        "Unknown".to_string()
    };

    let subtype = if path.is_dir() {
        WegItemSubtype::Folder
    } else if path.ends_with(".exe") {
        WegItemSubtype::App
    } else {
        WegItemSubtype::File
    };

    // todo add support to UWP for seelen rofi
    let mut data = PinnedWegItemData {
        id: uuid::Uuid::new_v4().to_string(),
        subtype,
        umid: None,
        display_name,
        path: path.clone(),
        is_dir: false,
        relaunch_program: path.to_string_lossy().to_string(),
        relaunch_args: None,
        relaunch_in: None,
        windows: vec![],
        pin_disabled: false,
    };

    if path.extension() == Some(OsStr::new("lnk")) {
        data.umid = WindowsApi::get_file_umid(&path).ok();
        let (program, arguments) = WindowsApi::resolve_lnk_target(&path)?;
        data.is_dir = program.is_dir();
        data.relaunch_program = program.to_string_lossy().to_string(); //
        data.relaunch_args = Some(RelaunchArguments::String(
            arguments.to_string_lossy().to_string(),
        ));

        if program.extension() == Some(OsStr::new("exe")) {
            data.subtype = WegItemSubtype::App;
        }
    }

    let guard = FULL_STATE.load();
    let mut items = guard.weg_items.clone();
    items.center.insert(0, WegItem::Pinned(data));
    items.sanitize();
    guard.write_weg_items(&items)?;
    Ok(())
}
