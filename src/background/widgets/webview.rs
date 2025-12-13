use seelen_core::state::{Widget, WidgetLoader};

use crate::{
    app::get_app_handle, error::Result, state::application::FULL_STATE, utils::WidgetWebviewLabel,
    widgets::WebviewArgs,
};

pub struct WidgetWebview(pub tauri::WebviewWindow);

impl WidgetWebview {
    pub fn create(widget: &Widget, label: &WidgetWebviewLabel) -> Result<Self> {
        let state = FULL_STATE.load();
        let title = widget.metadata.display_name.get(state.locale());

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

        let window: tauri::WebviewWindow =
            tauri::WebviewWindowBuilder::new(get_app_handle(), &label.raw, url)
                .title(title)
                .transparent(true)
                .visible(false)
                .data_directory(args.data_directory())
                .additional_browser_args(&args.to_string())
                .build()?;

        Ok(Self(window))
    }
}

impl Drop for WidgetWebview {
    fn drop(&mut self) {
        let _ = self.0.destroy();
    }
}
