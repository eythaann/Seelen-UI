use image::RgbaImage;
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

        Ok(Some(
            image::imageops::crop_imm(&raw, crop_x, crop_y, crop_w, crop_h).to_image(),
        ))
    }
}
