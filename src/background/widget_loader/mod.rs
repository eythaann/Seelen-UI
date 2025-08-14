use seelen_core::state::Widget;

use crate::{
    error_handler::Result, seelen::get_app_handle, state::application::FULL_STATE,
    utils::WidgetWebviewLabel,
};

pub struct WidgetInstance {
    /// main window label
    label: WidgetWebviewLabel,
    /// main window
    window: tauri::WebviewWindow,
    /// this will be filled only if widget instances is set to multiple
    extra_instances: Vec<(WidgetWebviewLabel, tauri::WebviewWindow)>,
}

impl Drop for WidgetInstance {
    fn drop(&mut self) {
        log::info!("Dropping {:?}", self.label.decoded);
        let _ = self.window.destroy();

        for (label, window) in self.extra_instances.drain(..) {
            log::info!("Dropping {}", label.decoded);
            let _ = window.destroy();
        }
    }
}

impl WidgetInstance {
    pub fn load(widget: Widget, monitor_id: &str) -> Result<Self> {
        let label = WidgetWebviewLabel::new(&widget.id, Some(monitor_id), None);

        let state = FULL_STATE.load();
        let title = widget.metadata.display_name.get(state.locale());
        let window = Self::create_window(title, &label)?;

        let mut extra_instances = vec![];
        for ins in state.get_widget_instances_ids(&widget.id) {
            let label = WidgetWebviewLabel::new(&widget.id, Some(monitor_id), Some(&ins));
            let window = Self::create_window(title, &label)?;
            extra_instances.push((label, window));
        }

        Ok(Self {
            label,
            window,
            extra_instances,
        })
    }

    fn create_window(title: &str, label: &WidgetWebviewLabel) -> Result<tauri::WebviewWindow> {
        let window = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            &label.raw,
            tauri::WebviewUrl::App("third_party/index.html".into()),
        )
        .title(title)
        .transparent(true)
        .visible(false)
        .build()?;
        Ok(window)
    }
}
