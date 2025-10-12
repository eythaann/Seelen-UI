pub mod cli;

use tauri::WebviewWindow;

use crate::{
    app::get_app_handle,
    error::{ErrorMap, Result, ResultLogExt},
    utils::WidgetWebviewLabel,
    widgets::{TrustedWidget, WebviewArgs},
};

pub struct TaskSwitcher {
    label: WidgetWebviewLabel,
    webview: WebviewWindow,
}

impl TrustedWidget for TaskSwitcher {
    const ID: &str = "@seelen/task-switcher";

    fn title() -> String {
        "Task Switcher".to_string()
    }
}

impl Drop for TaskSwitcher {
    fn drop(&mut self) {
        log::info!("Dropping '{}'", self.label.decoded);
        self.webview.destroy().wrap_error().log_error();
    }
}

impl TaskSwitcher {
    pub fn new() -> Result<Self> {
        let label = WidgetWebviewLabel::new(Self::ID, None, None);
        let args = WebviewArgs::new().disable_gpu();

        let webview = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            &label.raw,
            tauri::WebviewUrl::App("task_switcher/index.html".into()),
        )
        .title(Self::title())
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .resizable(false)
        .visible(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .drag_and_drop(false)
        .disable_drag_drop_handler()
        .always_on_top(true)
        .data_directory(args.data_directory())
        .additional_browser_args(&args.to_string())
        .build()?;

        Ok(Self { label, webview })
    }
}
