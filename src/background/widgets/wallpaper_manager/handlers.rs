use crate::{error::Result, widgets::webview};

impl crate::tauri_handlers::Handlers {
    pub fn set_as_wallpaper(webview: tauri::WebviewWindow) -> Result<()> {
        super::attach_to_desktop(&webview::native_window(&webview)?)
    }
}
