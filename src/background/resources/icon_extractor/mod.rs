//! Extraction of the app/file icons shown by the widgets, stored on the system icon pack.
//!
//! The queue, failure cache and image post-processing are OS-agnostic; how an icon is obtained
//! from a file or an app id is native (`imp`).

pub mod queue;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::path::Path;

use image::{GenericImageView, RgbaImage};

use queue::{IconExtractor, IconExtractorRequest};

use crate::error::Result;

const SQUARE_MARGIN: f32 = 0.1;
const ASPECT_TOLERANCE: f32 = 0.05;
const OPACITY_THRESHOLD: u8 = 254;

/// this is the best solution having in consideration that a transparent image and have separated pixels
/// with transparent gaps, so search side by side and crop them is the best approach.
pub fn crop_transparent_borders(rgba_image: &RgbaImage) -> RgbaImage {
    let (width, height) = rgba_image.dimensions();
    let mut top = None;
    let mut bottom = None;
    let mut left = None;
    let mut right = None;

    'outer: for y in 0..height {
        for x in 0..width {
            let pixel = rgba_image.get_pixel(x, y);
            if pixel.0[3] != 0 {
                top = Some(y);
                break 'outer;
            }
        }
    }

    let top = match top {
        Some(top) => top,
        None => return RgbaImage::new(1, 1),
    };

    'outer: for y in (top..height).rev() {
        for x in 0..width {
            let pixel = rgba_image.get_pixel(x, y);
            if pixel.0[3] != 0 {
                bottom = Some(y);
                break 'outer;
            }
        }
    }

    let bottom = match bottom {
        Some(bottom) => bottom,
        None => return RgbaImage::new(1, 1),
    };

    'outer: for x in 0..width {
        for y in top..bottom {
            let pixel = rgba_image.get_pixel(x, y);
            if pixel.0[3] != 0 {
                left = Some(x);
                break 'outer;
            }
        }
    }

    let left = match left {
        Some(left) => left,
        None => return RgbaImage::new(1, 1),
    };

    'outer: for x in (left..width).rev() {
        for y in top..bottom {
            let pixel = rgba_image.get_pixel(x, y);
            if pixel.0[3] != 0 {
                right = Some(x);
                break 'outer;
            }
        }
    }

    let right = match right {
        Some(right) => right,
        None => return RgbaImage::new(1, 1),
    };

    rgba_image
        .view(left, top, right - left + 1, bottom - top + 1)
        .to_image()
}

pub fn request_icon_extraction_from_file<T: AsRef<Path>>(path: T) {
    let path = path.as_ref();
    if path.as_os_str().is_empty() {
        return;
    }
    IconExtractor::instance().request(IconExtractorRequest::Path(path.to_path_buf()));
}

/// `app_id` is the OS id of an installed app (Windows: app user model id).
pub fn request_icon_extraction_from_umid(app_id: &str) {
    IconExtractor::instance().request(IconExtractorRequest::AppUMID(app_id.to_owned()));
}

pub fn is_aproximately_a_square(rgba_image: &RgbaImage) -> bool {
    let (width, height) = rgba_image.dimensions();

    // verify if the image is not empty
    if width == 0 || height == 0 {
        return false;
    }

    // verify if the image is a square
    let aspect_ratio = width as f32 / height as f32;
    if (aspect_ratio - 1.0).abs() > ASPECT_TOLERANCE {
        return false;
    }

    // Calculate margin
    let margin_x = (width as f32 * SQUARE_MARGIN) as u32;
    let margin_y = (height as f32 * SQUARE_MARGIN) as u32;
    let inner_width = width - 2 * margin_x;
    let inner_height = height - 2 * margin_y;

    // verify if the image is a square
    for y in margin_y..margin_y + inner_height {
        for x in margin_x..margin_x + inner_width {
            let pixel = rgba_image.get_pixel(x, y);
            if pixel.0[3] < OPACITY_THRESHOLD {
                return false;
            }
        }
    }

    true
}

fn save_as_webp(image: &RgbaImage, path: &std::path::Path) -> Result<()> {
    let webp_bytes =
        webp::Encoder::from_rgba(image.as_raw(), image.width(), image.height()).encode_lossless();
    std::fs::write(path, &*webp_bytes)?;
    Ok(())
}
