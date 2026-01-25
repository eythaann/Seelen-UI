use base64::Engine;
use seelen_core::state::{FancyToolbarSide, HideMode, SeelenWegSide};
use tauri::{WebviewWindow, Wry};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SWP_NOSIZE},
};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    state::application::FULL_STATE,
    widgets::{toolbar::FancyToolbar, webview::WebviewArgs},
    windows_api::{monitor::Monitor, AppBarData, WindowsApi},
};

pub struct SeelenWeg {
    pub window: WebviewWindow<Wry>,
    /// This is the GUI rect of the dock, not used as webview window rect
    pub theoretical_rect: RECT,
    /// This is the webview/window rect
    pub webview_rect: RECT,
}

impl Drop for SeelenWeg {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        if let Ok(hwnd) = self.hwnd() {
            AppBarData::from_handle(hwnd).unregister_bar();
        }
        log_error!(self.window.destroy());
    }
}

impl SeelenWeg {
    pub const TITLE: &'static str = "SeelenWeg";
    pub const TARGET: &'static str = "@seelen/weg";

    pub fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.hwnd()?.0))
    }

    fn create_window(monitor_id: &str) -> Result<WebviewWindow> {
        let manager = get_app_handle();
        let label = format!("{}?monitorId={}", Self::TARGET, monitor_id);
        let args = WebviewArgs::new().disable_gpu();

        log::info!("Creating {label}");
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&label);

        let window = tauri::WebviewWindowBuilder::new(
            manager,
            label,
            tauri::WebviewUrl::App("react/weg/index.html".into()),
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

        window.set_ignore_cursor_events(true)?;
        Ok(window)
    }

    pub fn new(monitor_id: &str) -> Result<Self> {
        let weg = Self {
            window: Self::create_window(monitor_id)?,
            theoretical_rect: RECT::default(),
            webview_rect: RECT::default(),
        };
        SeelenWeg::hide_native_taskbar();
        Ok(weg)
    }

    pub fn get_weg_size_on_monitor(monitor: &Monitor) -> Result<i32> {
        let state = FULL_STATE.load();
        let settings: &seelen_core::state::SeelenWegSettings = &state.settings.by_widget.weg;
        let total_size = (settings.total_size() as f64 * monitor.scale_factor()?) as i32;
        Ok(total_size)
    }

    pub fn set_position(&mut self, monitor: &Monitor) -> Result<()> {
        let state = FULL_STATE.load();
        let toolbar_config = &state.settings.by_widget.fancy_toolbar;
        let is_toolbar_enabled = state.is_bar_enabled_on_monitor(&monitor.stable_id2()?);

        let settings = &state.settings.by_widget.weg;

        let hwnd = HWND(self.hwnd()?.0);
        let monitor_info = WindowsApi::monitor_info(monitor.handle())?;

        let mut work_area = monitor_info.monitorInfo.rcMonitor;
        if is_toolbar_enabled && toolbar_config.hide_mode != HideMode::Always {
            let toolbar_size = FancyToolbar::get_toolbar_height_on_monitor(monitor)?;
            match state.settings.by_widget.fancy_toolbar.position {
                FancyToolbarSide::Top => {
                    work_area.top += toolbar_size;
                }
                FancyToolbarSide::Bottom => {
                    work_area.bottom -= toolbar_size;
                }
            }
        }

        self.theoretical_rect = work_area;
        self.webview_rect = work_area;

        // note: we reduce by 10px the webview size of the dock to avoid be matched as a fullscreen window
        let dock_size = Self::get_weg_size_on_monitor(monitor)?;
        match settings.position {
            SeelenWegSide::Left => {
                self.theoretical_rect.right = self.theoretical_rect.left + dock_size;
                self.webview_rect.right = work_area.right - (work_area.right - work_area.left) / 2;
            }
            SeelenWegSide::Right => {
                self.theoretical_rect.left = self.theoretical_rect.right - dock_size;
                self.webview_rect.left = work_area.left + (work_area.right - work_area.left) / 2;
            }
            SeelenWegSide::Top => {
                self.theoretical_rect.bottom = self.theoretical_rect.top + dock_size;
                self.webview_rect.bottom = work_area.top + (work_area.bottom - work_area.top) / 2;
            }
            SeelenWegSide::Bottom => {
                self.theoretical_rect.top = self.theoretical_rect.bottom - dock_size;
                self.webview_rect.top = work_area.bottom - (work_area.bottom - work_area.top) / 2;
            }
        }

        let mut abd = AppBarData::from_handle(hwnd);
        match settings.hide_mode {
            HideMode::Never => {
                abd.set_edge(settings.position.into());
                abd.set_rect(self.theoretical_rect);
                abd.register_as_new_bar();
            }
            _ => abd.unregister_bar(),
        };

        // pre set position for resize in case of multiples dpi
        WindowsApi::set_position(hwnd, None, &self.webview_rect, SWP_NOSIZE)?;
        WindowsApi::set_position(hwnd, None, &self.webview_rect, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }

    pub fn reposition_if_needed(&mut self) -> Result<()> {
        let hwnd = self.hwnd()?;
        if self.webview_rect == WindowsApi::get_outer_window_rect(hwnd)? {
            return Ok(()); // position is ok no need to reposition
        }
        self.set_position(&Monitor::from(WindowsApi::monitor_from_window(hwnd)))?;
        Ok(())
    }
}
