pub mod cli;
pub mod hook;

use crate::{
    app::get_app_handle,
    error_handler::Result,
    log_error,
    state::application::FULL_STATE,
    utils::{
        are_overlaped,
        constants::{NATIVE_UI_POPUP_CLASSES, OVERLAP_BLACK_LIST_BY_EXE},
    },
    windows_api::{window::Window, AppBarData, WindowsApi},
};
use base64::Engine;
use seelen_core::{
    handlers::SeelenEvent,
    state::{FancyToolbarSide, HideMode},
};
use serde::Serialize;
use tauri::{Emitter, WebviewWindow};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SW_HIDE, SW_SHOWNOACTIVATE},
};

pub struct FancyToolbar {
    window: WebviewWindow,
    /// Is the rect that the toolbar should have when it isn't hidden
    theoretical_rect: RECT,
    overlaped_by: Option<Window>,
    hidden: bool,
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

    pub fn new(monitor: &str) -> Result<Self> {
        Ok(Self {
            window: Self::create_window(monitor)?,
            theoretical_rect: RECT::default(),
            overlaped_by: None,
            hidden: false,
        })
    }

    pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    fn is_overlapping(&self, window: &Window) -> Result<bool> {
        let window_rect = WindowsApi::get_inner_window_rect(window.hwnd())?;
        Ok(are_overlaped(&self.theoretical_rect, &window_rect))
    }

    pub fn set_overlaped(&mut self, overlaped_by: Option<Window>) -> Result<()> {
        if self.overlaped_by != overlaped_by {
            self.emit(SeelenEvent::ToolbarOverlaped, overlaped_by.is_some())?;
        }
        self.overlaped_by = overlaped_by;
        let is_fullscreen = self.overlaped_by.is_some_and(|w| w.is_fullscreen());
        if is_fullscreen {
            self.hide()?;
        } else {
            self.show()?;
        }
        Ok(())
    }

    pub fn handle_overlaped_status(&mut self, window: &Window) -> Result<()> {
        let is_overlaped = self.is_overlapping(window)?
            && !window.is_desktop()
            && !window.is_seelen_overlay()
            && !NATIVE_UI_POPUP_CLASSES.contains(&window.class().as_str())
            && !OVERLAP_BLACK_LIST_BY_EXE.contains(
                &window
                    .process()
                    .program_exe_name()
                    .unwrap_or_default()
                    .as_str(),
            );

        if is_overlaped {
            return self.set_overlaped(Some(*window));
        }

        if self.overlaped_by.is_some()
            && WindowsApi::monitor_from_window(self.hwnd()?) == window.monitor().handle()
        {
            self.set_overlaped(None)?;
        }
        Ok(())
    }

    pub fn hide(&mut self) -> Result<()> {
        if self.hidden {
            return Ok(());
        }
        WindowsApi::show_window_async(self.hwnd()?, SW_HIDE)?;
        self.hidden = true;
        self.window.emit_to(
            self.window.label(),
            SeelenEvent::HandleLayeredHitboxes,
            false,
        )?;
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        if !self.hidden {
            return Ok(());
        }
        WindowsApi::show_window_async(self.hwnd()?, SW_SHOWNOACTIVATE)?;
        self.hidden = false;
        self.window.emit_to(
            self.window.label(),
            SeelenEvent::HandleLayeredHitboxes,
            true,
        )?;
        Ok(())
    }
}

// statics
impl FancyToolbar {
    pub const TITLE: &'static str = ".Seelen Fancy Toolbar";
    pub const TARGET: &'static str = "@seelen/fancy-toolbar";

    pub fn decoded_label(monitor_id: &str) -> String {
        format!("{}?monitorId={}", Self::TARGET, monitor_id)
    }

    pub fn label(monitor_id: &str) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Self::decoded_label(monitor_id))
    }

    /// Work area no works fine on multiple monitors
    /// so we use this functions that only takes the toolbar in account
    pub fn get_work_area_by_monitor(monitor: HMONITOR) -> Result<RECT> {
        let state = FULL_STATE.load();
        let settings = &state.settings.by_widget.fancy_toolbar;

        let monitor_info = WindowsApi::monitor_info(monitor)?;
        let monitor_scale_factor = WindowsApi::get_monitor_scale_factor(monitor)?;
        let text_scale_factor = WindowsApi::get_text_scale_factor()?;
        let real_height =
            (settings.height as f64 * monitor_scale_factor * text_scale_factor) as i32;

        let mut rect = monitor_info.monitorInfo.rcMonitor;
        if state.is_bar_enabled() {
            match settings.position {
                FancyToolbarSide::Top => {
                    rect.top += real_height;
                }
                FancyToolbarSide::Bottom => {
                    rect.bottom -= real_height;
                }
            }
        }

        Ok(rect)
    }

    pub fn set_position(&mut self, monitor: HMONITOR) -> Result<()> {
        let hwnd = HWND(self.hwnd()?.0);

        let state = FULL_STATE.load();
        let settings = &state.settings.by_widget.fancy_toolbar;

        let monitor_info = WindowsApi::monitor_info(monitor)?;
        let monitor_scale_factor = WindowsApi::get_monitor_scale_factor(monitor)?;
        let text_scale_factor = WindowsApi::get_text_scale_factor()?;
        let rc_monitor = monitor_info.monitorInfo.rcMonitor;

        let real_height =
            (settings.height as f64 * monitor_scale_factor * text_scale_factor) as i32;
        self.theoretical_rect = rc_monitor;
        match settings.position {
            FancyToolbarSide::Top => {
                self.theoretical_rect.bottom = rc_monitor.top + real_height;
            }
            FancyToolbarSide::Bottom => {
                self.theoretical_rect.top = rc_monitor.bottom - real_height;
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
        WindowsApi::move_window(hwnd, &rc_monitor)?;
        WindowsApi::set_position(hwnd, None, &rc_monitor, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }

    fn create_window(monitor_id: &str) -> Result<WebviewWindow> {
        let manager = get_app_handle();

        log::info!("Creating {}", Self::decoded_label(monitor_id));

        let window = tauri::WebviewWindowBuilder::new(
            manager,
            Self::label(monitor_id),
            tauri::WebviewUrl::App("toolbar/index.html".into()),
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
        .build()?;
        window.set_ignore_cursor_events(true)?;
        Ok(window)
    }
}
