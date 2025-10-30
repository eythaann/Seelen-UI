use seelen_core::state::{Widget, WidgetLoader};

use crate::{
    app::get_app_handle, error::Result, state::application::FULL_STATE, utils::WidgetWebviewLabel,
    widgets::WebviewArgs,
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
            log::info!("Dropping {:?}", label.decoded);
            let _ = window.destroy();
        }
    }
}

impl WidgetInstance {
    pub fn load(widget: &Widget, monitor_id: Option<&str>) -> Result<Self> {
        let label = WidgetWebviewLabel::new(&widget.id, monitor_id, None);
        log::info!("Creating {:?}", label.decoded);

        let state = FULL_STATE.load();
        let title = widget.metadata.display_name.get(state.locale());
        let window = Self::create_window(widget, title, &label)?;

        let mut extra_instances = vec![];
        for ins in state.get_widget_instances_ids(&widget.id) {
            let label = WidgetWebviewLabel::new(&widget.id, monitor_id, Some(&ins));
            let window = Self::create_window(widget, title, &label)?;
            extra_instances.push((label, window));
        }

        let instance = Self {
            label,
            window,
            extra_instances,
        };

        Ok(instance)
    }

    fn create_window(
        widget: &Widget,
        title: &str,
        label: &WidgetWebviewLabel,
    ) -> Result<tauri::WebviewWindow> {
        let args = WebviewArgs::new().disable_gpu();

        let url = match widget.loader {
            WidgetLoader::Legacy => {
                return Err("Legacy widgets are not supported by the new widget loader".into());
            }
            WidgetLoader::Internal => {
                let resource_name = widget.id.resource_name();
                tauri::WebviewUrl::App(format!("svelte/{resource_name}/index.html").into())
            }
            WidgetLoader::ThirdParty => {
                tauri::WebviewUrl::App("vanilla/third_party/index.html".into())
            }
        };

        let window = tauri::WebviewWindowBuilder::new(get_app_handle(), &label.raw, url)
            .title(title)
            .transparent(true)
            .visible(false)
            .data_directory(args.data_directory())
            .additional_browser_args(&args.to_string())
            .build()?;

        Ok(window)
    }
}
