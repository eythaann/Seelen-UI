mod live_previews;
mod window_peek;

use std::{
    sync::Once,
    time::{Duration, Instant},
};

use image::RgbaImage;
use seelen_core::{
    rect::Rect,
    system_state::{LiveWindowPreviewState, WindowPeekStatus, WindowPreviewTarget},
};
use win_screenshot::prelude::capture_window;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};

use crate::{
    error::Result,
    platform::{
        hook::{HookManager, WindowEvent},
        window::{Window, WindowsWindowExt},
    },
    system_modules::app_previews::domain::{AppPreviewsSystem, PreviewsSysEvent},
    windows_api::WindowsApi,
};

/// Stateless: the hook subscriptions last for the whole app lifetime.
pub struct WindowsAppPreviews;

impl AppPreviewsSystem for WindowsAppPreviews {
    fn create() -> Self {
        Self
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(PreviewsSysEvent) + Send + Sync + 'static,
    {
        HookManager::subscribe(move |(event, window)| {
            let hwnd = window.address();
            match event {
                WindowEvent::TitleChanged | WindowEvent::RectChanged => {
                    on_event(PreviewsSysEvent::WindowChanged { hwnd })
                }
                WindowEvent::Unminimized => on_event(PreviewsSysEvent::WindowRestored { hwnd }),
                _ => {}
            }
        });
        Ok(())
    }

    fn clear_native_previews(&self) {
        live_previews::LivePreviewManager::clear_if_initialized();
        window_peek::WindowPeekManager::clear_if_initialized();
    }

    fn update_live(
        &self,
        owner: isize,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
        anchor: Option<Rect>,
    ) -> Result<LiveWindowPreviewState> {
        start_preview_watchdog();
        live_previews::LivePreviewManager::instance()
            .update(owner, session, targets, opacity, anchor)
    }
    fn clear_live(&self, owner: isize, session: &str) {
        live_previews::LivePreviewManager::instance().clear(owner, session);
    }
    fn begin_peek(
        &self,
        owner: isize,
        hwnd: isize,
        session: String,
        rect: Rect,
    ) -> Result<WindowPeekStatus> {
        start_preview_watchdog();
        window_peek::WindowPeekManager::instance().begin(owner, hwnd, session, rect)
    }
    fn poll_peek(&self, owner: isize, session: &str, rect: Option<Rect>) -> WindowPeekStatus {
        window_peek::WindowPeekManager::instance().poll(owner, session, rect)
    }
    fn cancel_peek(&self, owner: isize, session: &str) {
        window_peek::WindowPeekManager::instance().cancel(owner, session);
    }
    fn commit_peek(&self, owner: isize, session: &str) -> Result<bool> {
        window_peek::WindowPeekManager::instance().commit(owner, session)
    }

    fn capture(&self, hwnd: isize) -> Result<Option<RgbaImage>> {
        let window = Window::from(hwnd);
        if !window.is_manageable_from_unelevated() || window.is_minimized() {
            return Ok(None);
        }

        let buf = capture_window(hwnd).map_err(|_| "Failed to capture window")?;
        let raw = RgbaImage::from_raw(buf.width, buf.height, buf.pixels)
            .ok_or("Failed to create image")?;

        let shadow = WindowsApi::shadow_rect(window.hwnd())?;
        let crop_x = shadow.left.unsigned_abs();
        let crop_y = shadow.top.unsigned_abs();
        let crop_w = buf
            .width
            .saturating_sub(shadow.left.unsigned_abs() + shadow.right.unsigned_abs());
        let crop_h = buf
            .height
            .saturating_sub(shadow.top.unsigned_abs() + shadow.bottom.unsigned_abs());

        if crop_w == 0 || crop_h == 0 {
            return Ok(None);
        }

        Ok(Some(
            image::imageops::crop_imm(&raw, crop_x, crop_y, crop_w, crop_h).to_image(),
        ))
    }
}

/// One lazy watchdog drives both leases without initializing an unused native manager.
fn start_preview_watchdog() {
    static START: Once = Once::new();
    START.call_once(|| {
        std::thread::Builder::new()
            .name("AppPreviewWatchdog".into())
            .spawn(|| {
                let mut last_cleanup = Instant::now();
                let mut escape_was_down = false;
                loop {
                    std::thread::sleep(Duration::from_millis(32));
                    // Hover popups deliberately keep focus in the user's application.
                    let escape_down = unsafe { GetAsyncKeyState(VK_ESCAPE.0 as i32) } < 0;
                    if escape_down && !escape_was_down {
                        live_previews::LivePreviewManager::request_dismiss_if_initialized();
                    }
                    escape_was_down = escape_down;
                    window_peek::WindowPeekManager::tick_if_initialized();
                    if last_cleanup.elapsed() >= Duration::from_millis(150) {
                        live_previews::LivePreviewManager::cleanup_if_initialized();
                        last_cleanup = Instant::now();
                    }
                }
            })
            .expect("app preview watchdog");
    });
}
