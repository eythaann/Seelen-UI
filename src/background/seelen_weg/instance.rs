use base64::Engine;
use seelen_core::{
    handlers::SeelenEvent,
    state::{HideMode, SeelenWegSide},
};
use serde::Serialize;
use tauri::{Emitter, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SW_HIDE, SW_SHOWNOACTIVATE},
};

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    seelen_bar::FancyToolbar,
    state::application::FULL_STATE,
    utils::{
        are_overlaped,
        constants::{NATIVE_UI_POPUP_CLASSES, OVERLAP_BLACK_LIST_BY_EXE},
    },
    windows_api::{window::Window, AppBarData, WindowsApi},
};

pub struct SeelenWeg {
    pub window: WebviewWindow<Wry>,
    pub overlaped: bool,
    pub last_overlapped_window: Option<Window>,
    /// Is the rect that the dock should have when it isn't hidden
    pub theoretical_rect: RECT,
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

    pub fn get_label(monitor_id: &str) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(format!(
            "{}?monitor={}",
            Self::TARGET,
            monitor_id
        ))
    }

    fn create_window(postfix: &str) -> Result<WebviewWindow> {
        let manager = get_app_handle();

        let label = format!("{}?monitor={}", Self::TARGET, postfix);
        log::info!("Creating {}", label);
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&label);

        let window = tauri::WebviewWindowBuilder::new(
            manager,
            label,
            tauri::WebviewUrl::App("seelenweg/index.html".into()),
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

    pub fn new(postfix: &str) -> Result<Self> {
        let weg = Self {
            window: Self::create_window(postfix)?,
            overlaped: false,
            last_overlapped_window: None,
            theoretical_rect: RECT::default(),
        };
        Ok(weg)
    }

    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    fn is_overlapping(&self, hwnd: HWND) -> Result<bool> {
        let window_rect = WindowsApi::get_inner_window_rect(hwnd)?;
        Ok(are_overlaped(&self.theoretical_rect, &window_rect))
    }

    fn set_overlaped_status(&mut self, is_overlaped: bool) -> Result<()> {
        if self.overlaped == is_overlaped {
            return Ok(());
        }
        self.overlaped = is_overlaped;
        self.emit(SeelenEvent::WegOverlaped, self.overlaped)?;
        Ok(())
    }

    pub fn handle_overlaped_status(&mut self, hwnd: HWND) -> Result<()> {
        let window = Window::from(hwnd);
        let monitor = window.monitor();
        let is_overlaped = self.is_overlapping(hwnd)?
            && !window.is_desktop()
            && !window.is_seelen_overlay()
            && !NATIVE_UI_POPUP_CLASSES.contains(&window.class().as_str())
            && !OVERLAP_BLACK_LIST_BY_EXE
                .contains(&WindowsApi::exe(hwnd).unwrap_or_default().as_str());

        let state = FULL_STATE.load();
        let settings = state.settings().seelenweg();

        if settings.use_multi_monitor_overlap_logic {
            if is_overlaped {
                self.last_overlapped_window = Some(window);
            } else if let Some(past_window) = self.last_overlapped_window {
                if past_window != window
                    && past_window.monitor() != monitor
                    && Window::from(self.hwnd()?).monitor() != monitor
                {
                    return Ok(());
                }
            }
        } else {
            self.last_overlapped_window = None;
        }

        self.set_overlaped_status(is_overlaped)
    }

    pub fn hide(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.hwnd()?, SW_HIDE)?;
        self.window.emit_to(
            self.window.label(),
            SeelenEvent::HandleLayeredHitboxes,
            false,
        )?;
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.hwnd()?, SW_SHOWNOACTIVATE)?;
        self.window.emit_to(
            self.window.label(),
            SeelenEvent::HandleLayeredHitboxes,
            true,
        )?;
        Ok(())
    }

    pub fn set_position(&mut self, monitor: HMONITOR) -> Result<()> {
        let rc_work = FancyToolbar::get_work_area_by_monitor(monitor)?;
        let hwnd = HWND(self.hwnd()?.0);

        let state = FULL_STATE.load();
        let settings = state.settings().seelenweg();
        let monitor_dpi = WindowsApi::get_device_pixel_ratio(monitor)?;
        let total_size = (settings.total_size() as f32 * monitor_dpi) as i32;

        self.theoretical_rect = rc_work;
        let mut hidden_rect = rc_work;
        match settings.position {
            SeelenWegSide::Left => {
                self.theoretical_rect.right = self.theoretical_rect.left + total_size;
                hidden_rect.right = hidden_rect.left + 1;
            }
            SeelenWegSide::Right => {
                self.theoretical_rect.left = self.theoretical_rect.right - total_size;
                hidden_rect.left = hidden_rect.right - 1;
            }
            SeelenWegSide::Top => {
                self.theoretical_rect.bottom = self.theoretical_rect.top + total_size;
                hidden_rect.bottom = hidden_rect.top + 1;
            }
            SeelenWegSide::Bottom => {
                self.theoretical_rect.top = self.theoretical_rect.bottom - total_size;
                hidden_rect.top = hidden_rect.bottom - 1;
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
        WindowsApi::move_window(hwnd, &rc_work)?;
        WindowsApi::set_position(hwnd, None, &rc_work, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }
}
