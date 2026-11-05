use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{DynamicImage, RgbaImage};
use seelen_core::system_state::{Color, UserAppWindowColors, UserAppWindowPreview};

use crate::error::Result;

/// Number of pixel samples taken evenly across each edge of a window.
const SAMPLING: usize = 7;
const WEBP_QUALITY: f32 = 75.0;

/// OS contract of the app windows previews module.
pub trait AppPreviewsSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Starts listening the window changes that can change their content, reported through
    /// `on_event` for every window (filtering them is up to the caller).
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> crate::error::Result<()>
    where
        F: Fn(PreviewsSysEvent) + Send + Sync + 'static;

    /// Captures the content of the window, without its frame shadow. `None` when it can't
    /// be captured now (minimized, or not accessible from this process).
    fn capture(&self, hwnd: isize) -> Result<Option<RgbaImage>>;

    /// False while the screen is locked or another user is active.
    fn is_session_interactive(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewsSysEvent {
    /// The title or placement changed, so probably its content too.
    WindowChanged { hwnd: isize },
    /// The window was restored, its content is drawn a bit later.
    WindowRestored { hwnd: isize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewsEvent {
    Captured { hwnd: isize },
    ColorsUpdated { hwnd: isize },
    Cleaned { hwnd: isize },
}

/// Scales down big captures, as they are only shown as thumbnails.
pub fn shrink_capture(image: RgbaImage) -> RgbaImage {
    let (w, h) = image.dimensions();
    if w > 1024 || h > 1024 {
        image::imageops::thumbnail(&image, w / 2, h / 2)
    } else {
        image
    }
}

pub fn encode_preview(image: RgbaImage, hash: String) -> Result<UserAppWindowPreview> {
    let dynamic = DynamicImage::ImageRgba8(image);
    let webp_bytes = webp::Encoder::from_image(&dynamic)
        .map_err(|e| e.to_string())?
        .encode(WEBP_QUALITY);
    Ok(UserAppWindowPreview {
        hash,
        data: STANDARD.encode(&*webp_bytes),
        width: dynamic.width(),
        height: dynamic.height(),
    })
}

/// Samples SAMPLING pixels evenly spaced along each edge.
pub fn sample_edge_colors(image: &RgbaImage) -> UserAppWindowColors {
    let w = image.width();
    let h = image.height();

    let sample = |i: usize, len: u32| -> u32 {
        if SAMPLING <= 1 {
            return 0;
        }
        ((i as u64 * (len as u64 - 1)) / (SAMPLING as u64 - 1)) as u32
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

    UserAppWindowColors {
        top: (0..SAMPLING)
            .map(|i| pixel_to_color(sample(i, w), 0))
            .collect(),
        bottom: (0..SAMPLING)
            .map(|i| pixel_to_color(sample(i, w), h.saturating_sub(1)))
            .collect(),
        left: (0..SAMPLING)
            .map(|i| pixel_to_color(0, sample(i, h)))
            .collect(),
        right: (0..SAMPLING)
            .map(|i| pixel_to_color(w.saturating_sub(1), sample(i, h)))
            .collect(),
    }
}

pub fn image_hash(image: &RgbaImage) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    image.as_raw().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
