use base64::Engine;
use seelen_core::state::Widget;

use crate::{error_handler::Result, seelen::get_app_handle};

#[allow(dead_code)]
pub struct WidgetInstance {
    widget: Widget,
    window: tauri::WebviewWindow,
}

impl WidgetInstance {
    pub fn load(widget: Widget) -> Result<Self> {
        let window = Self::create_window(&widget)?;
        Ok(Self { widget, window })
    }

    fn create_window(widget: &Widget) -> Result<tauri::WebviewWindow> {
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&widget.id);

        let window = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            label,
            tauri::WebviewUrl::App("widget_loader/index.html".into()),
        )
        .title(format!("{} - Seelen UI Widget", &widget.id))
        .build()?;
        Ok(window)
    }
}
