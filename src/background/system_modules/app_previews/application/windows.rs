mod live_previews;

use std::{
    sync::{Arc, OnceLock, Weak},
    thread::Thread,
    time::{Duration, Instant},
};

use image::RgbaImage;
use seelen_core::{
    rect::Rect,
    system_state::{LiveWindowPreviewState, WindowPreviewTarget},
};
use win_screenshot::prelude::capture_window;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_ESCAPE};

use crate::{
    error::{Result, ResultLogExt},
    platform::{
        hook::{HookManager, WindowEvent},
        window::{Window, WindowsWindowExt},
    },
    system_modules::app_previews::domain::{AppPreviewsSystem, PreviewsSysEvent},
    windows_api::WindowsApi,
};

/// Escape polling cadence while a preview is on screen.
const WATCHDOG_TICK: Duration = Duration::from_millis(50);
/// Expired live leases are released at this cadence.
const LEASE_CLEANUP: Duration = Duration::from_millis(150);

/// The hook subscriptions last for the whole app lifetime. Live surfaces are owned here and
/// released through `clear_native_previews` on exit.
pub struct WindowsAppPreviews {
    native: Arc<NativePreviews>,
}

impl AppPreviewsSystem for WindowsAppPreviews {
    fn create() -> Self {
        Self {
            native: Arc::new(NativePreviews {
                live: live_previews::LivePreviewManager::create(),
                watchdog: OnceLock::new(),
            }),
        }
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
        self.native.live.clear_all();
    }

    fn update_live(
        &self,
        owner: isize,
        session: String,
        targets: Vec<WindowPreviewTarget>,
        opacity: u8,
        anchor: Option<Rect>,
    ) -> Result<LiveWindowPreviewState> {
        let status = self
            .native
            .live
            .update(owner, session, targets, opacity, anchor);
        // Wake after the lease exists, otherwise the watchdog could go back to sleep.
        self.native.wake();
        status
    }
    fn clear_live(&self, owner: isize, session: &str) {
        self.native.live.clear(owner, session);
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

/// Live surfaces, plus the watchdog that expires their leases when the webview stops polling.
struct NativePreviews {
    live: live_previews::LivePreviewManager,
    /// Spawned on first use. `None` if the thread could not be created.
    watchdog: OnceLock<Option<Thread>>,
}

impl NativePreviews {
    /// Must be called after a lease was created: an idle watchdog parks until woken.
    fn wake(self: &Arc<Self>) {
        let watchdog = self.watchdog.get_or_init(|| {
            let native = Arc::downgrade(self);
            std::thread::Builder::new()
                .name("app-preview-watchdog".into())
                .spawn(move || watch(native))
                .ok_logged()
                .map(|handle| handle.thread().clone())
        });
        if let Some(thread) = watchdog {
            thread.unpark();
        }
    }

    fn is_idle(&self) -> bool {
        self.live.is_idle()
    }
}

/// Parks while nothing is on screen, so an unused dock preview costs no wakeups.
fn watch(native: Weak<NativePreviews>) {
    let mut last_cleanup = Instant::now();
    let mut escape_was_down = false;
    loop {
        let Some(native) = native.upgrade() else {
            return;
        };
        if native.is_idle() {
            drop(native);
            escape_was_down = false;
            // An `unpark` issued before this point makes `park` return immediately.
            std::thread::park();
            continue;
        }

        // Hover popups deliberately keep focus in the user's application, so Escape is polled.
        let escape_down = unsafe { GetAsyncKeyState(VK_ESCAPE.0 as i32) } < 0;
        if escape_down && !escape_was_down {
            native.live.request_dismiss();
        }
        escape_was_down = escape_down;
        if last_cleanup.elapsed() >= LEASE_CLEANUP {
            native.live.cleanup();
            last_cleanup = Instant::now();
        }
        drop(native);
        std::thread::sleep(WATCHDOG_TICK);
    }
}
