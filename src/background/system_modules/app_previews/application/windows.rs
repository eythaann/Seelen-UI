pub(crate) mod live_previews;
pub(crate) mod window_peek;

use image::RgbaImage;
use seelen_core::{
    rect::Rect,
    system_state::{LiveWindowPreviewState, WindowPeekStatus, WindowPreviewTarget},
};
use win_screenshot::prelude::capture_window;

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

    fn update_live(
        &self,
        owner: isize,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
        anchor: Option<Rect>,
    ) -> Result<LiveWindowPreviewState> {
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
