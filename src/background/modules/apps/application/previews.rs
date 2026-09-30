use std::{collections::HashMap, sync::LazyLock, time::Duration};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{DynamicImage, RgbaImage};
use seelen_core::system_state::{Color, UserAppWindowColors, UserAppWindowPreview};
use slu_utils::{Debounce, debounce};
use win_screenshot::prelude::capture_window;

use super::graphics_capture;
use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    hook::HookManager,
    modules::apps::application::{USER_APPS_MANAGER, UserAppWinEvent, UserAppsManager},
    utils::lock_free::SyncHashMap,
    windows_api::{
        WindowsApi,
        event_window::IS_INTERACTIVE_SESSION,
        window::{Window, event::WinEvent},
    },
};

const CAPTURE_WINDOW_INTERVAL: Duration = Duration::from_millis(200);
/// Number of pixel samples taken evenly across the top and bottom edges of a window.
const SAMPLING: usize = 7;

static WINDOWS_PREVIEWS: LazyLock<WinPreviewManager> = LazyLock::new(WinPreviewManager::create);

#[derive(Clone, Copy)]
struct CaptureRequest {
    addr: isize,
    capture_preview: bool,
}

/// One-by-one capture queue: sender side, receiver processes in a dedicated thread.
static CAPTURE_TX: LazyLock<crossbeam_channel::Sender<CaptureRequest>> = LazyLock::new(|| {
    let (tx, rx) = crossbeam_channel::unbounded::<CaptureRequest>();
    std::thread::Builder::new()
        .name("win-capture-queue".into())
        .spawn(move || {
            for request in rx {
                if !IS_INTERACTIVE_SESSION.load(std::sync::atomic::Ordering::Acquire) {
                    continue;
                }

                let window = Window::from(request.addr);
                if let Err(error) = WINDOWS_PREVIEWS.do_capture(&window, request.capture_preview) {
                    log::debug!(
                        "Failed to capture window ({:x}): {error:?}",
                        request.addr
                    );
                }
            }
        })
        .log_error();
    tx
});

struct UserAppWindowPreviewWrap {
    preview: Option<UserAppWindowPreview>,
    colors: Option<UserAppWindowColors>,
    capture: Debounce<()>,
    refresh_generation: u64,
}

pub struct WinPreviewManager {
    previews: SyncHashMap<isize, UserAppWindowPreviewWrap>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WinPreviewEvent {
    Captured(isize),
    ColorsUpdated(isize),
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
                let window = Window::from(addr);
                if window.is_window() && (window.is_minimized() || !window.is_visible()) {
                    WINDOWS_PREVIEWS.suspend_window(addr);
                } else {
                    WINDOWS_PREVIEWS.remove_window(addr);
                }
            }
        });

        HookManager::subscribe(|(event, window)| {
            if event == WinEvent::ObjectDestroy {
                WINDOWS_PREVIEWS.remove_window(window.address());
                return;
            }

            if !USER_APPS_MANAGER.contains_win(&window) {
                return;
            }
            let addr = window.address();
            match event {
                WinEvent::ObjectNameChange | WinEvent::SynDebouncedRectChange => {
                    WINDOWS_PREVIEWS.enqueue_capture_burst(addr);
                }
                WinEvent::SystemMinimizeEnd => {
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(300));
                        WINDOWS_PREVIEWS.enqueue_capture_burst(addr);
                    });
                }
                _ => {}
            }
        });

        Ok(())
    }

    fn register_window(&self, addr: isize) {
        if self.previews.contains_key(&addr) {
            self.enqueue_capture_burst(addr);
            return;
        }

        let capture = debounce(
            move |_| {
                if let Err(e) = CAPTURE_TX.send(CaptureRequest {
                    addr,
                    capture_preview: true,
                }) {
                    log::error!("Failed to enqueue capture: {e}");
                }
            },
            CAPTURE_WINDOW_INTERVAL,
        );
        self.previews.upsert(
            addr,
            UserAppWindowPreviewWrap {
                preview: None,
                colors: None,
                capture,
                refresh_generation: 0,
            },
        );
        self.enqueue_capture_burst(addr);
    }

    fn enqueue_capture(&self, addr: isize) {
        self.previews.get(&addr, |wrap| {
            wrap.capture.call(());
        });
    }

    fn enqueue_capture_burst(&self, addr: isize) {
        self.enqueue_capture(addr);

        let Some(generation) = self.previews.get(&addr, |wrap| {
            wrap.refresh_generation = wrap.refresh_generation.wrapping_add(1);
            wrap.refresh_generation
        }) else {
            return;
        };

        std::thread::spawn(move || {
            for delay in [250, 450, 800] {
                std::thread::sleep(Duration::from_millis(delay));

                let is_current = WINDOWS_PREVIEWS
                    .previews
                    .get(&addr, |wrap| wrap.refresh_generation == generation)
                    .unwrap_or(false);
                let window = Window::from(addr);
                if !is_current
                    || !window.is_window()
                    || window.is_minimized()
                    || !window.is_focused()
                    || !window.is_maximized()
                {
                    break;
                }

                if CAPTURE_TX
                    .send(CaptureRequest {
                        addr,
                        capture_preview: false,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
    }

    fn do_capture(&self, window: &Window, capture_preview: bool) -> Result<()> {
        if window.is_minimized() {
            return Ok(());
        }

        let addr = window.address();
        log::trace!("capturing window ({addr:x})");

        let (raw, has_shadow) = match capture_window(window.address()) {
            Ok(buf) => (
                RgbaImage::from_raw(buf.width, buf.height, buf.pixels)
                    .ok_or("Failed to create image")?,
                true,
            ),
            Err(error) => {
                log::debug!(
                    "PrintWindow failed for {addr:x}; trying Windows Graphics Capture: {error:?}"
                );
                (graphics_capture::capture_window(addr)?, false)
            }
        };

        let cropped: RgbaImage = if has_shadow {
            let shadow = WindowsApi::shadow_rect(window.hwnd())?;
            let crop_x = shadow.left.unsigned_abs();
            let crop_y = shadow.top.unsigned_abs();
            let crop_w = raw
                .width()
                .saturating_sub(shadow.left.unsigned_abs() + shadow.right.unsigned_abs());
            let crop_h = raw
                .height()
                .saturating_sub(shadow.top.unsigned_abs() + shadow.bottom.unsigned_abs());
            image::imageops::crop_imm(&raw, crop_x, crop_y, crop_w, crop_h).to_image()
        } else {
            raw
        };
        let crop_w = cropped.width();
        let crop_h = cropped.height();
        let image: RgbaImage = if capture_preview && (crop_w > 1024 || crop_h > 1024) {
            image::imageops::thumbnail(&cropped, crop_w / 2, crop_h / 2)
        } else {
            cropped
        };

        let colors = sample_edge_colors(&image);

        if capture_preview {
            let image_hash = image_to_hash(&image);
            let unchanged = self
                .previews
                .get(&addr, |wrap| {
                    wrap
                        .preview
                        .as_ref()
                        .map(|p| p.hash == image_hash)
                        .unwrap_or(false)
                })
                .unwrap_or(true);

            if !unchanged {
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
            }
        }

        self.update_colors(addr, colors);

        Ok(())
    }

    fn update_colors(&self, addr: isize, colors: UserAppWindowColors) {
        let changed = self
            .previews
            .get(&addr, |wrap| {
                let changed = wrap
                    .colors
                    .as_ref()
                    .map(|current| !same_window_colors(current, &colors))
                    .unwrap_or(true);
                if changed {
                    wrap.colors = Some(colors);
                }
                changed
            })
            .unwrap_or(false);

        if changed {
            Self::send(WinPreviewEvent::ColorsUpdated(addr));
        }
    }

    fn suspend_window(&self, addr: isize) {
        let had_preview = self
            .previews
            .get(&addr, |wrap| {
                wrap.refresh_generation = wrap.refresh_generation.wrapping_add(1);
                wrap.preview.take().is_some()
            })
            .unwrap_or(false);
        if had_preview {
            Self::send(WinPreviewEvent::Cleaned(addr));
        }
    }

    fn remove_window(&self, addr: isize) {
        if self.previews.remove(&addr).is_some() {
            Self::send(WinPreviewEvent::Cleaned(addr));
            Self::send(WinPreviewEvent::ColorsUpdated(addr));
        }
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

    pub fn get_colors(&self) -> HashMap<isize, UserAppWindowColors> {
        let mut map = HashMap::new();
        self.previews.for_each(|(k, v)| {
            if let Some(colors) = &v.colors {
                map.insert(*k, colors.clone());
            }
        });
        map
    }
}

fn same_window_colors(left: &UserAppWindowColors, right: &UserAppWindowColors) -> bool {
    let same_edge = |left: &[Color], right: &[Color]| {
        left.len() == right.len()
            && left.iter().zip(right).all(|(left, right)| {
                left.r == right.r
                    && left.g == right.g
                    && left.b == right.b
                    && left.a == right.a
            })
    };

    same_edge(&left.top, &right.top)
        && same_edge(&left.bottom, &right.bottom)
        && same_edge(&left.left, &right.left)
        && same_edge(&left.right, &right.right)
}

/// Samples SAMPLING pixels evenly spaced from left to right along the top and bottom rows.
fn sample_edge_colors(image: &RgbaImage) -> UserAppWindowColors {
    let w = image.width();
    let h = image.height();

    let sample_x = |i: usize| -> u32 {
        if SAMPLING <= 1 {
            return 0;
        }
        ((i as u64 * (w as u64 - 1)) / (SAMPLING as u64 - 1)) as u32
    };

    let pixel_to_color = |x: u32, y: u32| -> Color {
        let p = image.get_pixel(x, y);
        Color {
            r: p[0],
            g: p[1],
            b: p[2],
            a: p[3],
        }
    };

    let sample_y = |i: usize| -> u32 {
        if SAMPLING <= 1 {
            return 0;
        }
        ((i as u64 * (h as u64 - 1)) / (SAMPLING as u64 - 1)) as u32
    };

    let top = (0..SAMPLING)
        .map(|i| pixel_to_color(sample_x(i), 0))
        .collect();
    let bottom = (0..SAMPLING)
        .map(|i| pixel_to_color(sample_x(i), h.saturating_sub(1)))
        .collect();
    let left = (0..SAMPLING)
        .map(|i| pixel_to_color(0, sample_y(i)))
        .collect();
    let right = (0..SAMPLING)
        .map(|i| pixel_to_color(w.saturating_sub(1), sample_y(i)))
        .collect();

    UserAppWindowColors {
        top,
        bottom,
        left,
        right,
    }
}

fn image_to_hash(icon_image: &image::RgbaImage) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    icon_image.as_raw().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
