use tauri::{App, AppHandle, WebviewWindow};

use crate::{error_handler::Result, SEELEN};

pub fn show_settings_window(app: &AppHandle) -> Result<WebviewWindow> {
    let window = tauri::WebviewWindowBuilder::new(
        app,
        "settings",
        tauri::WebviewUrl::App("settings/index.html".into()),
    )
    .inner_size(700.0, 500.0)
    .maximizable(false)
    .minimizable(true)
    .resizable(false)
    .title("Komorebi UI - Settings")
    .visible(false)
    .decorations(false)
    .center()
    .build()?;

    Ok(window)
}

pub fn show_seelenpad_window(app: &AppHandle) -> Result<WebviewWindow> {
    let window = tauri::WebviewWindowBuilder::new(
        app,
        "seelenpad",
        tauri::WebviewUrl::App("seelenpad/index.html".into()),
    )
    .inner_size(300.0, 300.0)
    .maximizable(false)
    .minimizable(false)
    .resizable(false)
    .title("Seelenpad")
    .visible(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .build()?;

    Ok(window)
}

pub fn set_windows_events(app: &mut App) -> Result<()> {
    app.listen("open_settings", |_| {
        show_settings_window(SEELEN.lock().handle()).ok();
    });

    app.listen("open_seelenpad", |_| {
        show_seelenpad_window(SEELEN.lock().handle()).ok();
    });

    Ok(())
}
