use tauri::Manager;

use crate::{error_handler::Result, seelen::get_app_handle, utils::WidgetWebviewLabel};

pub fn show_settings() -> Result<()> {
    log::trace!("Show settings window");
    let label = WidgetWebviewLabel::new("@seelen/settings", None, None);
    let handle = get_app_handle();
    match handle.get_webview_window(&label.raw) {
        Some(window) => {
            window.unminimize()?;
            window.set_focus()?;
        }
        None => {
            tauri::WebviewWindowBuilder::new(
                handle,
                label.raw,
                tauri::WebviewUrl::App("settings/index.html".into()),
            )
            .title("Settings")
            .inner_size(800.0, 500.0)
            .min_inner_size(600.0, 400.0)
            .visible(false)
            .decorations(false)
            .center()
            .build()?;
        }
    }
    Ok(())
}
