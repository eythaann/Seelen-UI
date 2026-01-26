pub mod hook;

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    state::application::FULL_STATE,
    widgets::webview::WebviewArgs,
    windows_api::{monitor::Monitor, AppBarData, WindowsApi},
};
use base64::Engine;
use seelen_core::state::{FancyToolbarSide, HideMode};
use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SWP_NOSIZE},
};

pub struct FancyToolbar {
    window: WebviewWindow,
    pub rect: RECT,
}

impl Drop for FancyToolbar {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        if let Ok(hwnd) = self.hwnd() {
            AppBarData::from_handle(hwnd).unregister_bar();
        }
        log_error!(self.window.destroy());
    }
}

impl FancyToolbar {
    pub fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.hwnd()?.0))
    }

    pub fn new(monitor_id: &str) -> Result<Self> {
        Ok(Self {
            window: Self::create_window(monitor_id)?,
            rect: RECT::default(),
        })
    }
}

// statics
impl FancyToolbar {
    pub const TITLE: &'static str = "Seelen Fancy Toolbar";
    pub const TARGET: &'static str = "@seelen/fancy-toolbar";

    pub fn decoded_label(monitor_id: &str) -> String {
        format!("{}?monitorId={}", Self::TARGET, monitor_id)
    }

    pub fn label(monitor_id: &str) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Self::decoded_label(monitor_id))
    }

    fn create_window(monitor_id: &str) -> Result<WebviewWindow> {
        let manager = get_app_handle();
        let args = WebviewArgs::new().disable_gpu();

        log::info!("Creating {}", Self::decoded_label(monitor_id));

        let window = tauri::WebviewWindowBuilder::new(
            manager,
            Self::label(monitor_id),
            tauri::WebviewUrl::App("react/toolbar/index.html".into()),
        )
        .title(Self::TITLE)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .resizable(false)
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .data_directory(args.data_directory())
        .additional_browser_args(&args.to_string())
        .build()?;

        Ok(window)
    }

    pub fn get_toolbar_height_on_monitor(monitor: &Monitor) -> Result<i32> {
        let state = FULL_STATE.load();
        let settings = &state.settings.by_widget.fancy_toolbar;
        let scale_factor = monitor.scale_factor()?;
        Ok((settings.height as f64 * scale_factor) as i32)
    }

    pub fn set_position(&mut self, monitor: &Monitor) -> Result<()> {
        let hwnd = HWND(self.hwnd()?.0);

        let state = FULL_STATE.load();
        let settings = &state.settings.by_widget.fancy_toolbar;

        let monitor_info = WindowsApi::monitor_info(monitor.handle())?;
        let rc_monitor = monitor_info.monitorInfo.rcMonitor;

        let real_height = Self::get_toolbar_height_on_monitor(monitor)?;

        self.rect = rc_monitor;
        match settings.position {
            FancyToolbarSide::Top => {
                self.rect.bottom = rc_monitor.top + real_height;
            }
            FancyToolbarSide::Bottom => {
                self.rect.top = rc_monitor.bottom - real_height;
            }
        }

        let mut abd = AppBarData::from_handle(hwnd);
        match settings.hide_mode {
            HideMode::Never => {
                abd.set_edge(settings.position.into());
                abd.set_rect(self.rect);
                abd.register_as_new_bar();
            }
            _ => abd.unregister_bar(),
        };

        // pre set position for resize in case of multiples dpi
        WindowsApi::set_position(hwnd, None, &self.rect, SWP_NOSIZE)?;
        WindowsApi::set_position(hwnd, None, &self.rect, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }

    pub fn reposition_if_needed(&mut self) -> Result<()> {
        let hwnd = self.hwnd()?;
        if self.rect == WindowsApi::get_outer_window_rect(hwnd)? {
            return Ok(()); // position is ok no need to reposition
        }
        self.set_position(&Monitor::from(WindowsApi::monitor_from_window(hwnd)))?;
        Ok(())
    }
}
