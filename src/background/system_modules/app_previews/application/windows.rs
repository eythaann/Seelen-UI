mod live_previews;

use image::RgbaImage;
use seelen_core::system_state::{LiveWindowPreviewState, WindowHandle, WindowPreviewTarget};
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

/// The hook subscriptions last for the whole app lifetime. Live surfaces are owned here and
/// released through `clear_native_previews` on exit.
pub struct WindowsAppPreviews {
    live: live_previews::LivePreviewManager,
}

impl AppPreviewsSystem for WindowsAppPreviews {
    fn create() -> Self {
        Self {
            live: live_previews::LivePreviewManager::create(),
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(PreviewsSysEvent) + Send + Sync + 'static,
    {
        HookManager::subscribe(move |(event, window)| {
            let hwnd = window.handle();
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
        self.live.clear_all();
    }

    fn update_live(
        &self,
        owner: WindowHandle,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
    ) -> Result<LiveWindowPreviewState> {
        self.live.update(owner, session, targets, opacity)
    }

    fn clear_live(&self, owner: WindowHandle, session: &str) {
        self.live.clear(owner, session);
    }

    fn capture(&self, hwnd: WindowHandle) -> Result<Option<RgbaImage>> {
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
