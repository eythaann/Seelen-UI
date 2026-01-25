use std::path::PathBuf;

use base64::Engine;
use seelen_core::{
    resource::WidgetId,
    state::{Widget, WidgetLoader},
};

use crate::{
    app::get_app_handle, error::Result, state::application::FULL_STATE,
    utils::constants::SEELEN_COMMON,
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
            WidgetLoader::InternalReact => {
                let resource_name = widget
                    .id
                    .resource_name()
                    .ok_or("Can't get internal resource path")?;
                tauri::WebviewUrl::App(format!("react/{resource_name}/index.html").into())
            }
            WidgetLoader::Internal => {
                let resource_name = widget
                    .id
                    .resource_name()
                    .ok_or("Can't get internal resource path")?;
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

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WidgetWebviewLabel {
    /// this should be used as the real webview label
    pub raw: String,
    /// this is the decoded label, useful for debugging and logging
    decoded: String,
    /// widget id from this label was created
    pub widget_id: WidgetId,
}

impl WidgetWebviewLabel {
    pub fn new(
        widget_id: &WidgetId,
        monitor_id: Option<&str>,
        instance_id: Option<&uuid::Uuid>,
    ) -> Self {
        let mut label = widget_id.to_string();
        let with_monitor_id = monitor_id.is_some();
        let with_instance_id = instance_id.is_some();
        if with_monitor_id || with_instance_id {
            label.push('?');
        }

        if let Some(monitor_id) = monitor_id {
            label.push_str(&format!("monitorId={}", urlencoding::encode(monitor_id)));
        }

        if let Some(instance_id) = instance_id {
            if with_monitor_id {
                label.push('&');
            }
            label.push_str(&format!(
                "instanceId={}",
                urlencoding::encode(&instance_id.to_string())
            ));
        }

        Self {
            raw: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&label),
            decoded: label,
            widget_id: widget_id.clone(),
        }
    }

    pub fn try_from_raw(raw: &str) -> Result<Self> {
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(raw)?;
        let decoded = String::from_utf8(decoded)?;
        let widget_id = WidgetId::from(decoded.split('?').next().expect("Invalid label"));

        Ok(Self {
            raw: raw.to_string(),
            decoded,
            widget_id,
        })
    }
}

impl std::fmt::Display for WidgetWebviewLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.decoded)
    }
}

// =============================================================================

pub struct WebviewArgs {
    common_args: Vec<String>,
    extra_args: Vec<String>,
}

impl WebviewArgs {
    const BASE_OPT: &str = "--disable-features=translate,msWebOOUI,msPdfOOUI,msSmartScreenProtection,RendererAppContainer";
    const BASE_OPT2: &str =
        "--no-first-run --disable-site-isolation-trials --disable-background-timer-throttling";
    const PERFORMANCE_OPT: &str = "--enable-low-end-device-mode --in-process-gpu --V8Maglev";

    pub fn new() -> Self {
        Self {
            common_args: vec![
                Self::BASE_OPT.to_string(),
                Self::BASE_OPT2.to_string(),
                Self::PERFORMANCE_OPT.to_string(),
            ],
            extra_args: vec![],
        }
    }

    pub fn disable_gpu(self) -> Self {
        // if window manager is enabled (that is expected thing) having 2 processes one with gpu and another without,
        // is worse than having them together with gpu enabled so this is the reason why this is currently ignored.
        // self.extra_args.push("--disable-gpu --disable-software-rasterizer".to_string());
        self
    }

    pub fn data_directory(&self) -> PathBuf {
        if self.extra_args.is_empty() {
            SEELEN_COMMON.app_cache_dir().to_path_buf()
        } else {
            SEELEN_COMMON
                .app_cache_dir()
                .join(self.extra_args.join("").replace("-", ""))
        }
    }
}

impl std::fmt::Display for WebviewArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.common_args.join(" "))
    }
}
