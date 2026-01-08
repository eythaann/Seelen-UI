use std::sync::LazyLock;

use image::{DynamicImage, RgbaImage};
use seelen_core::system_state::UserAppWindowPreview;
use win_screenshot::prelude::capture_window;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    modules::apps::application::{UserAppWinEvent, UserAppsManager},
    utils::{constants::SEELEN_COMMON, lock_free::SyncHashMap},
    windows_api::{window::Window, WindowsApi},
};

static WINDOWS_PREVIEWS: LazyLock<WinPreviewManager> = LazyLock::new(WinPreviewManager::create);

pub struct WinPreviewManager {
    pub previews: SyncHashMap<isize, UserAppWindowPreview>,
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
        let windows = UserAppsManager::instance()
            .interactable_windows
            .map(|w| w.hwnd);

        std::thread::spawn(move || {
            for hwnd in windows {
                let window = Window::from(hwnd);
                WINDOWS_PREVIEWS.capture_window(&window).log_error();
            }
        });

        UserAppsManager::subscribe(|e| match e {
            UserAppWinEvent::Added(addr) => {
                let window = Window::from(addr);
                WINDOWS_PREVIEWS.capture_window(&window).log_error();
            }
            UserAppWinEvent::Updated(addr) => {
                let window = Window::from(addr);
                WINDOWS_PREVIEWS.capture_window(&window).log_error();
            }
            UserAppWinEvent::Removed(addr) => {
                if let Some(preview) = WINDOWS_PREVIEWS.previews.remove(&addr) {
                    if preview.path.exists() {
                        std::fs::remove_file(&preview.path).log_error();
                    }
                    Self::send(WinPreviewEvent::Cleaned(addr));
                }
            }
        });
        Ok(())
    }

    fn capture_window(&self, window: &Window) -> Result<()> {
        if window.is_minimized() {
            return Ok(());
        }

        let addr = window.address();

        let buf = capture_window(window.address()).map_err(|_| "Failed to capture window")?;
        let image = RgbaImage::from_raw(buf.width, buf.height, buf.pixels)
            .ok_or("Failed to create image")?;

        let image_hash = image_to_hash(&image);
        let image = DynamicImage::ImageRgba8(image);

        let box_shadow = WindowsApi::shadow_rect(window.hwnd())?;
        let image = image.crop_imm(
            box_shadow.left.unsigned_abs(),
            box_shadow.top.unsigned_abs(),
            buf.width - (box_shadow.left + box_shadow.right).unsigned_abs(),
            buf.height - (box_shadow.top + box_shadow.bottom).unsigned_abs(),
        );

        let path_to_save = SEELEN_COMMON.app_temp_dir().join(format!("{addr}.webp"));
        image.save_with_format(&path_to_save, image::ImageFormat::WebP)?;

        self.previews.upsert(
            addr,
            UserAppWindowPreview {
                hash: image_hash,
                path: path_to_save,
                width: image.width(),
                height: image.height(),
            },
        );
        Self::send(WinPreviewEvent::Captured(addr));
        Ok(())
    }
}

fn image_to_hash(icon_image: &image::RgbaImage) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    icon_image.as_raw().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
