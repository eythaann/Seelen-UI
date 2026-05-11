use std::{collections::HashMap, sync::LazyLock, time::Duration};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{DynamicImage, RgbaImage};
use seelen_core::system_state::UserAppWindowPreview;
use slu_utils::{debounce, Debounce};
use win_screenshot::prelude::capture_window;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    hook::HookManager,
    modules::apps::application::{UserAppWinEvent, UserAppsManager, USER_APPS_MANAGER},
    utils::lock_free::SyncHashMap,
    windows_api::{
        window::{event::WinEvent, Window},
        WindowsApi,
    },
};

const CAPTURE_WINDOW_INTERVAL: Duration = Duration::from_millis(200);
static WINDOWS_PREVIEWS: LazyLock<WinPreviewManager> = LazyLock::new(WinPreviewManager::create);

/// One-by-one capture queue: sender side, receiver processes in a dedicated thread.
static CAPTURE_TX: LazyLock<crossbeam_channel::Sender<isize>> = LazyLock::new(|| {
    let (tx, rx) = crossbeam_channel::unbounded::<isize>();
    std::thread::Builder::new()
        .name("win-capture-queue".into())
        .spawn(move || {
            for addr in rx {
                let window = Window::from(addr);
                WINDOWS_PREVIEWS.do_capture(&window).log_error();
            }
        })
        .log_error();
    tx
});

struct UserAppWindowPreviewWrap {
    preview: Option<UserAppWindowPreview>,
    capture: Debounce<()>,
}

pub struct WinPreviewManager {
    previews: SyncHashMap<isize, UserAppWindowPreviewWrap>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WinPreviewEvent {
    Captured(isize),
    Cleaned(isize),
}

event_manager!(WinPreviewManager, WinPreviewEvent);

impl WinPreviewManager {
    pub fn instance() -> &'static Self {
        &WINDOWS_PREVIEWS
    }

    fn create() -> Self {
        let manager = Self {
            previews: SyncHashMap::new(),
        };
        manager.init().log_error();
        manager
    }

    fn init(&self) -> Result<()> {
        // Force initialization of the capture queue thread before any enqueue.
        let _ = &*CAPTURE_TX;

        let windows = UserAppsManager::instance()
            .interactable_windows
            .map(|w| w.hwnd);

        for hwnd in windows {
            self.register_window(hwnd);
        }

        UserAppsManager::subscribe(|e| match e {
            UserAppWinEvent::Added(addr) => {
                WINDOWS_PREVIEWS.register_window(addr);
            }
            UserAppWinEvent::Updated(_) => {}
            UserAppWinEvent::Removed(addr) => {
                if WINDOWS_PREVIEWS.previews.remove(&addr).is_some() {
                    Self::send(WinPreviewEvent::Cleaned(addr));
                }
            }
        });

        HookManager::subscribe(|(event, window)| {
            if !USER_APPS_MANAGER.contains_win(&window) {
                return;
            }
            let addr = window.address();
            match event {
                WinEvent::ObjectNameChange | WinEvent::SynDebouncedForegroundRectChange => {
                    WINDOWS_PREVIEWS.enqueue_capture(addr);
                }
                WinEvent::SystemMinimizeEnd => {
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(300));
                        WINDOWS_PREVIEWS.enqueue_capture(addr);
                    });
                }
                _ => {}
            }
        });

        Ok(())
    }

    fn register_window(&self, addr: isize) {
        let capture = debounce(
            move |_| {
                if let Err(e) = CAPTURE_TX.send(addr) {
                    log::error!("Failed to enqueue capture: {e}");
                }
            },
            CAPTURE_WINDOW_INTERVAL,
        );
        self.previews.upsert(
            addr,
            UserAppWindowPreviewWrap {
                preview: None,
                capture,
            },
        );
        self.enqueue_capture(addr);
    }

    fn enqueue_capture(&self, addr: isize) {
        self.previews.get(&addr, |wrap| {
            wrap.capture.call(());
        });
    }

    fn do_capture(&self, window: &Window) -> Result<()> {
        if window.is_minimized() {
            return Ok(());
        }

        let addr = window.address();
        log::trace!("capturing window ({addr:x})");

        let buf = capture_window(window.address()).map_err(|_| "Failed to capture window")?;
        let raw = RgbaImage::from_raw(buf.width, buf.height, buf.pixels)
            .ok_or("Failed to create image")?;

        let shadow = WindowsApi::shadow_rect(window.hwnd())?;
        let crop_x = shadow.left.unsigned_abs();
        let crop_y = shadow.top.unsigned_abs();
        let crop_w = buf
            .width
            .saturating_sub((shadow.left + shadow.right).unsigned_abs());
        let crop_h = buf
            .height
            .saturating_sub((shadow.top + shadow.bottom).unsigned_abs());

        let cropped: RgbaImage =
            image::imageops::crop_imm(&raw, crop_x, crop_y, crop_w, crop_h).to_image();
        let image: RgbaImage = if crop_w > 1024 || crop_h > 1024 {
            image::imageops::thumbnail(&cropped, crop_w / 2, crop_h / 2)
        } else {
            cropped
        };

        let image_hash = image_to_hash(&image);

        let mut unchanged = false;
        self.previews.get(&addr, |wrap| {
            unchanged = wrap
                .preview
                .as_ref()
                .map(|p| p.hash == image_hash)
                .unwrap_or(false);
        });
        if unchanged {
            return Ok(());
        }

        let dynamic = DynamicImage::ImageRgba8(image);
        let webp_bytes = webp::Encoder::from_image(&dynamic)
            .map_err(|e| e.to_string())?
            .encode(75.0);
        let data = STANDARD.encode(&*webp_bytes);

        self.previews.get(&addr, |wrap| {
            wrap.preview = Some(UserAppWindowPreview {
                hash: image_hash,
                data,
                width: dynamic.width(),
                height: dynamic.height(),
            });
        });
        Self::send(WinPreviewEvent::Captured(addr));
        Ok(())
    }

    pub fn get_previews(&self) -> HashMap<isize, UserAppWindowPreview> {
        let mut map = HashMap::new();
        self.previews.for_each(|(k, v)| {
            if let Some(preview) = &v.preview {
                map.insert(*k, preview.clone());
            }
        });
        map
    }
}

fn image_to_hash(icon_image: &image::RgbaImage) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    icon_image.as_raw().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
