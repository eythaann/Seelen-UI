use base64::Engine;
use seelen_core::{
    handlers::SeelenEvent,
    state::{FancyToolbarSide, HideMode, SeelenWegSide},
};
use serde::Serialize;
use tauri::{Emitter, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{SWP_ASYNCWINDOWPOS, SW_HIDE, SW_SHOWNOACTIVATE},
};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    state::application::FULL_STATE,
    utils::{
        are_overlaped,
        constants::{NATIVE_UI_POPUP_CLASSES, OVERLAP_BLACK_LIST_BY_EXE},
    },
    widgets::{toolbar::FancyToolbar, WebviewArgs},
    windows_api::{window::Window, AppBarData, WindowsApi},
};

pub struct SeelenWeg {
    pub window: WebviewWindow<Wry>,
    /// This is the GUI rect of the dock, not used as webview window rect
    pub theoretical_rect: RECT,
    /// This is the webview/window rect
    pub webview_rect: RECT,
    pub overlaped_by: Option<Window>,
    pub hidden: bool,
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
            tauri::WebviewUrl::App("weg/index.html".into()),
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

    pub fn new(postfix: &str) -> Result<Self> {
        let weg = Self {
            window: Self::create_window(postfix)?,
            overlaped_by: None,
            theoretical_rect: RECT::default(),
            webview_rect: RECT::default(),
            hidden: false,
        };
        Ok(weg)
    }

    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    fn is_overlapping(&self, window: &Window) -> Result<bool> {
        let window_rect = WindowsApi::get_inner_window_rect(window.hwnd())?;
        Ok(are_overlaped(&self.theoretical_rect, &window_rect))
    }

    pub fn set_overlaped(&mut self, overlaped_by: Option<Window>) -> Result<()> {
        if self.overlaped_by != overlaped_by {
            self.emit(SeelenEvent::WegOverlaped, overlaped_by.is_some())?;
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

    pub fn get_weg_size_on_monitor(monitor: HMONITOR) -> Result<i32> {
        let state = FULL_STATE.load();
        let settings = &state.settings.by_widget.weg;
        let monitor_dpi = WindowsApi::get_monitor_scale_factor(monitor)?;
        let text_scale_factor = WindowsApi::get_text_scale_factor()?;
        let total_size = (settings.total_size() as f64 * monitor_dpi * text_scale_factor) as i32;
        Ok(total_size)
    }

    pub fn set_position(&mut self, monitor: HMONITOR) -> Result<()> {
        let state = FULL_STATE.load();
        let toolbar_config = &state.settings.by_widget.fancy_toolbar;
        let settings = &state.settings.by_widget.weg;

        let hwnd = HWND(self.hwnd()?.0);
        let monitor_info = WindowsApi::monitor_info(monitor)?;

        self.theoretical_rect = monitor_info.monitorInfo.rcMonitor;
        self.webview_rect = monitor_info.monitorInfo.rcMonitor;

        if toolbar_config.enabled && toolbar_config.hide_mode != HideMode::Always {
            let toolbar_size = FancyToolbar::get_toolbar_height_on_monitor(monitor)?;
            match state.settings.by_widget.fancy_toolbar.position {
                FancyToolbarSide::Top => {
                    self.webview_rect.top += toolbar_size;
                }
                FancyToolbarSide::Bottom => {
                    self.webview_rect.bottom -= toolbar_size;
                }
            }
        }

        // note: we reduce by 10px the webview size of the dock to avoid be matched as a fullscreen window
        let dock_size = Self::get_weg_size_on_monitor(monitor)?;
        match settings.position {
            SeelenWegSide::Left => {
                self.theoretical_rect.right = self.theoretical_rect.left + dock_size;
                self.webview_rect.right -= 10;
            }
            SeelenWegSide::Right => {
                self.theoretical_rect.left = self.theoretical_rect.right - dock_size;
                self.webview_rect.left += 10;
            }
            SeelenWegSide::Top => {
                self.theoretical_rect.bottom = self.theoretical_rect.top + dock_size;
                self.webview_rect.bottom -= 10;
            }
            SeelenWegSide::Bottom => {
                self.theoretical_rect.top = self.theoretical_rect.bottom - dock_size;
                self.webview_rect.top += 10;
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
        WindowsApi::move_window(hwnd, &self.webview_rect)?;
        WindowsApi::set_position(hwnd, None, &self.webview_rect, SWP_ASYNCWINDOWPOS)?;
        Ok(())
    }

    pub fn reposition_if_needed(&mut self) -> Result<()> {
        let hwnd = self.hwnd()?;
        if self.webview_rect == WindowsApi::get_outer_window_rect(hwnd)? {
            return Ok(()); // position is ok no need to reposition
        }
        self.set_position(WindowsApi::monitor_from_window(hwnd))?;
        Ok(())
    }
}
