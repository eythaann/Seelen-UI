use std::path::PathBuf;

use base64::Engine;
use seelen_core::{
    resource::WidgetId,
    state::{Widget, WidgetLoader, WidgetPreset},
    system_state::MonitorId,
};

use tauri::Manager;

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    state::application::FULL_STATE,
    utils::constants::SEELEN_COMMON,
};

pub struct WidgetWebview(pub tauri::WebviewWindow);

impl WidgetWebview {
    pub fn create(
        widget: &Widget,
        label: &WidgetWebviewLabel,
        owner_hwnd: Option<isize>,
    ) -> Result<Self> {
        let state = FULL_STATE.load();
        let title = widget.metadata.display_name.get(state.locale());

        let args = WebviewArgs::create(
            state.settings.hardware_acceleration || widget.id == WidgetId::known_wall(),
            state.settings.unstable_optimizations,
        );

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

        let mut builder = tauri::WebviewWindowBuilder::new(get_app_handle(), &label.raw, url)
            .title(title)
            .transparent(true)
            .visible(false);

        if matches!(
            widget.preset,
            WidgetPreset::Desktop | WidgetPreset::Overlay | WidgetPreset::Popup
        ) {
            builder = builder
                .decorations(false)
                .shadow(false)
                .skip_taskbar(true)
                .minimizable(false)
                .maximizable(false)
                .closable(false)
                .focusable(false);
        }

        match widget.preset {
            WidgetPreset::Desktop => {
                builder = builder.always_on_bottom(true);
            }
            WidgetPreset::Overlay | WidgetPreset::Popup => {
                builder = builder.always_on_top(true).resizable(false);
            }
            _ => {}
        }

        if let Some(owner) = owner_hwnd {
            // SAFETY: HWND in windows 0.61 (tauri) and 0.62 (ours) share the same memory layout
            #[allow(clippy::missing_transmute_annotations)]
            {
                builder = builder.owner_raw(unsafe { std::mem::transmute(owner) });
            }
        }

        let window = builder
            .data_directory(args.data_directory())
            .additional_browser_args(&args.to_string())
            .build()?;

        Ok(Self(window))
    }

    pub fn reload(&self) {
        self.0.reload().log_error();
    }
}

impl Drop for WidgetWebview {
    fn drop(&mut self) {
        // Only destroy if Tauri's manager still holds the window. When
        // WindowEvent::Destroyed fires, Tauri has already removed it from the
        // manager (get_webview_window returns None), so calling destroy() again
        // would cause re-entrant ZwUserDestroyWindow → FATAL_USER_CALLBACK_EXCEPTION.
        let label = self.0.label().to_owned();
        if get_app_handle().get_webview_window(&label).is_some() {
            let _ = self.0.destroy();
        }
    }
}

// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WidgetWebviewLabel {
    /// this should be used as the real webview label
    pub raw: String,
    /// this is the decoded label, useful for debugging and logging
    pub decoded: String,
    /// widget id from this label was created
    pub widget_id: WidgetId,
    pub monitor_id: Option<MonitorId>,
    pub instance_id: Option<uuid::Uuid>,
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
            monitor_id: monitor_id.map(MonitorId::from),
            instance_id: instance_id.cloned(),
        }
    }

    pub fn try_from_raw(raw: &str) -> Result<Self> {
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(raw)?;
        let decoded = String::from_utf8(decoded)?;

        let mut parts = decoded.splitn(2, '?');
        let widget_id = WidgetId::from(parts.next().expect("Invalid label"));

        let mut monitor_id = None;
        let mut instance_id = None;
        if let Some(query) = parts.next() {
            for param in query.split('&') {
                if let Some(value) = param.strip_prefix("monitorId=") {
                    let decoded_value = urlencoding::decode(value).unwrap_or_default();
                    monitor_id = Some(MonitorId::from(decoded_value.as_ref()));
                } else if let Some(value) = param.strip_prefix("instanceId=") {
                    let decoded_value = urlencoding::decode(value).unwrap_or_default();
                    instance_id = decoded_value.parse::<uuid::Uuid>().ok();
                }
            }
        }

        Ok(Self {
            raw: raw.to_string(),
            decoded,
            widget_id,
            monitor_id,
            instance_id,
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
    args: Vec<String>,
    with_gpu: bool,
    unstable: bool,
}

impl WebviewArgs {
    const BASE_ARGS: &[&str] = &[
        "--disable-features=translate,msWebOOUI,msPdfOOUI,msSmartScreenProtection,RendererAppContainer,BackForwardCache,InterestCohort,SharedArrayBuffer,CalculateNativeWinOcclusion,OptimizationHints,AutofillServerCommunication",
        "--no-first-run",
        "--disable-site-isolation-trials",
        "--disk-cache-size=0",
        "--media-cache-size=0",
        "--disable-background-networking",
        "--disable-background-timer-throttling",
        "--disable-backgrounding-occluded-windows",
        "--disable-renderer-backgrounding",
        "--disable-sync",
        "--disable-breakpad",
        "--disable-component-extensions-with-background-pages",
        "--no-pings",
        // "--aggressive-cache-discard", // maybe causes more resources than it reduces
    ];

    const GPU_ARGS: &[&str] = &[
        "--enable-gpu",
        "--enable-accelerated-video-decode",
        "--enable-gpu-rasterization",
        "--enable-zero-copy",
        "--enable-native-gpu-memory-buffers",
        "--enable-oop-rasterization",
        "--use-angle=d3d11", // Media Foundation + DXVA + D3D11 in windows is the most optimized
    ];

    const PERFORMANCE_ARGS: &[&str] = &[
        // "--enable-low-end-device-mode", // unstable flag that causes more issues than it solves
        // "--in-process-gpu", // unstable flag
        "--disable-gpu",
        "--disable-accelerated-video-decode",
        "--disable-accelerated-video-encode",
        "--disable-gpu-rasterization",
        "--disable-software-rasterizer",
    ];

    const UNSTABLE_OPTIMIZATIONS: &[&str] = &[
        // this reduces ram usage but if a widget crashes it will crash
        // all widgets with the same loader, so it's not worth it
        "--process-per-site",
    ];

    pub fn create(with_gpu: bool, unstable_optimizations: bool) -> Self {
        let mut args: Vec<String> = Self::BASE_ARGS.iter().map(|s| s.to_string()).collect();

        if with_gpu {
            args.extend(Self::GPU_ARGS.iter().map(|s| s.to_string()));
        } else {
            args.extend(Self::PERFORMANCE_ARGS.iter().map(|s| s.to_string()));
        };

        if unstable_optimizations {
            args.extend(Self::UNSTABLE_OPTIMIZATIONS.iter().map(|s| s.to_string()));
        }

        Self {
            args,
            with_gpu,
            unstable: unstable_optimizations,
        }
    }

    pub fn data_directory(&self) -> PathBuf {
        let foldername = match (self.with_gpu, self.unstable) {
            (true, true) => "gpu-unstable",
            (true, false) => "gpu",
            (false, true) => "no-gpu-unstable",
            (false, false) => "no-gpu",
        };

        SEELEN_COMMON.app_cache_dir().join(foldername)
    }
}

impl Default for WebviewArgs {
    fn default() -> Self {
        let state = FULL_STATE.load();
        Self::create(
            state.settings.hardware_acceleration,
            state.settings.unstable_optimizations,
        )
    }
}

impl std::fmt::Display for WebviewArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.args.join(" "))
    }
}
