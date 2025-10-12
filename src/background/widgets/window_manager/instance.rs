use base64::Engine;
use seelen_core::{
    state::{FancyToolbarSide, HideMode, SeelenWegSide},
    system_state::MonitorId,
};
use std::sync::Arc;
use tauri::{Listener, WebviewWindow};
use windows::Win32::{
    Foundation::HWND, Graphics::Gdi::HMONITOR, UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS,
};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    state::application::FULL_STATE,
    trace_lock,
    virtual_desktops::get_vd_manager,
    widgets::{
        toolbar::FancyToolbar, weg::SeelenWeg, window_manager::state::WM_STATE, WebviewArgs,
    },
    windows_api::WindowsApi,
};

pub struct WindowManagerV2 {
    pub window: WebviewWindow,
}

impl Drop for WindowManagerV2 {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        log_error!(self.window.destroy());
    }
}

impl WindowManagerV2 {
    pub const TITLE: &'static str = "Seelen Window Manager";
    pub const TARGET: &'static str = "@seelen/window-manager";

    pub fn new(monitor_id: &MonitorId) -> Result<Self> {
        Ok(Self {
            window: Self::create_window(monitor_id)?,
        })
    }

    pub fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.hwnd()?.0))
    }

    pub fn get_label(monitor_id: &str) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!(
            "{}?monitorId={}",
            Self::TARGET,
            monitor_id
        ))
    }

    fn create_window(monitor_id: &MonitorId) -> Result<WebviewWindow> {
        let label = format!("{}?monitorId={}", Self::TARGET, monitor_id);
        log::info!("Creating {label}");
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&label);
        let args = WebviewArgs::new().disable_gpu();

        let window = tauri::WebviewWindowBuilder::new(
            get_app_handle(),
            label,
            tauri::WebviewUrl::App("window_manager/index.html".into()),
        )
        .title(Self::TITLE)
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
        .always_on_top(true)
        .data_directory(args.data_directory())
        .additional_browser_args(&args.to_string())
        .build()?;

        window.set_ignore_cursor_events(true)?;

        let monitor_id = Arc::new(monitor_id.to_owned());

        window.listen("complete-setup", move |_event| {
            let monitor_id = monitor_id.clone();

            std::thread::spawn(move || -> Result<()> {
                let mut state = trace_lock!(WM_STATE);
                let workspace = state
                    .get_workspace_state(get_vd_manager().get_active_workspace_id(&monitor_id));
                Self::render_workspace(&monitor_id, workspace)?;
                Ok(())
            });
        });

        Ok(window)
    }

    pub fn set_position(&self, monitor: HMONITOR) -> Result<()> {
        let state = FULL_STATE.load();
        let toolbar_config = &state.settings.by_widget.fancy_toolbar;
        let weg_config = &state.settings.by_widget.weg;

        let hwnd = HWND(self.hwnd()?.0);
        let monitor_info = WindowsApi::monitor_info(monitor)?;

        let mut rect = monitor_info.monitorInfo.rcMonitor;
        if toolbar_config.enabled && toolbar_config.hide_mode != HideMode::Always {
            let toolbar_size = FancyToolbar::get_toolbar_height_on_monitor(monitor)?;
            match state.settings.by_widget.fancy_toolbar.position {
                FancyToolbarSide::Top => {
                    rect.top += toolbar_size;
                }
                FancyToolbarSide::Bottom => {
                    rect.bottom -= toolbar_size;
                }
            }
        }

        if weg_config.enabled && weg_config.hide_mode != HideMode::Always {
            let weg_size = SeelenWeg::get_weg_size_on_monitor(monitor)?;
            match weg_config.position {
                SeelenWegSide::Top => {
                    rect.top += weg_size;
                }
                SeelenWegSide::Bottom => {
                    rect.bottom -= weg_size;
                }
                SeelenWegSide::Left => {
                    rect.left += weg_size;
                }
                SeelenWegSide::Right => {
                    rect.right -= weg_size;
                }
            }
        }

        WindowsApi::move_window(hwnd, &rect)?;
        WindowsApi::set_position(hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }
}
