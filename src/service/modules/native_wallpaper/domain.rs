use serde::{Deserialize, Serialize};

use crate::error::Result;

/// OS contract of the native wallpaper module.
///
/// Used from a single worker thread, which creates and initializes it.
pub trait NativeWallpaperSystem: Sized {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Prepares the calling thread (the worker) to use the native wallpaper APIs.
    fn init(&mut self) -> Result<()>;

    /// (monitor id, wallpaper path) of each monitor known by the OS, an empty path means no
    /// wallpaper (solid color).
    fn wallpapers(&self) -> Result<Vec<(String, String)>>;

    /// The current configuration, `by_monitor` being the current `wallpapers()`.
    fn snapshot(&self, by_monitor: Vec<(String, String)>) -> Result<OriginalWallpaper>;

    /// Sets `path` on all the monitors, filling them.
    fn set_filled_wallpaper(&self, path: &str) -> Result<()>;

    /// Restores the position and background color of `original` (not the wallpapers).
    fn set_style(&self, original: &OriginalWallpaper) -> Result<()>;

    /// Starts the slideshow, fails if none of its items exist anymore.
    fn set_slideshow(&self, slideshow: &Slideshow) -> Result<()>;

    /// `monitor: None` sets it on all the monitors.
    fn set_wallpaper(&self, monitor: Option<&str>, wallpaper: &str) -> Result<()>;
}

/// Persisted while the native wallpaper is managed by Seelen UI.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedState {
    pub original: OriginalWallpaper,
    /// Last wallpaper set by us.
    pub applied: Option<String>,
}

impl SavedState {
    pub fn is_applied(&self, wallpaper: &str) -> bool {
        self.applied
            .as_deref()
            .is_some_and(|applied| paths_eq(applied, wallpaper))
    }
}

/// The user's wallpaper configuration before Seelen UI changed it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginalWallpaper {
    /// (monitor id, wallpaper path), an empty path means no wallpaper (solid color).
    pub by_monitor: Vec<(String, String)>,
    /// Native position (fill, fit, tile...) value, opaque outside the native layer.
    pub position: i32,
    /// Native color value, opaque outside the native layer.
    pub background_color: u32,
    /// `Some` if a slideshow was running.
    #[serde(default)]
    pub slideshow: Option<Slideshow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Slideshow {
    /// File system paths of the slideshow items (usually a single folder).
    pub items: Vec<String>,
    pub shuffle: bool,
    /// Milliseconds between images.
    pub interval: u32,
}

pub fn paths_eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// Files of Seelen UI (bundled or user resources, thumbnails, our black image) are never taken
/// as the user's own wallpaper, even if left by a previous session that didn't restore it.
pub fn is_seelen_owned(wallpaper: &str) -> bool {
    let lower = wallpaper.to_lowercase();
    if lower.contains("com.seelen.seelen-ui") || lower.contains("seelen.seelenui_") {
        return true;
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_string_lossy().to_lowercase()))
        .is_some_and(|dir| lower.starts_with(&dir))
}

/// 8x8 24 bits black bitmap (rows of 24 bytes, already 4-byte aligned).
pub fn black_bmp() -> Vec<u8> {
    const SIZE: u32 = 8;
    const DATA_SIZE: u32 = SIZE * SIZE * 3;
    const OFFSET: u32 = 14 + 40;

    let mut bmp = Vec::with_capacity((OFFSET + DATA_SIZE) as usize);
    // BITMAPFILEHEADER
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(OFFSET + DATA_SIZE).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&OFFSET.to_le_bytes());
    // BITMAPINFOHEADER
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(SIZE as i32).to_le_bytes());
    bmp.extend_from_slice(&(SIZE as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&24u16.to_le_bytes()); // bits per pixel
    bmp.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    bmp.extend_from_slice(&DATA_SIZE.to_le_bytes());
    bmp.extend_from_slice(&2835i32.to_le_bytes()); // 72 DPI
    bmp.extend_from_slice(&2835i32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    // pixels
    bmp.resize((OFFSET + DATA_SIZE) as usize, 0);
    bmp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_bmp_is_well_formed() {
        let bmp = black_bmp();
        assert_eq!(&bmp[0..2], b"BM");
        assert_eq!(
            u32::from_le_bytes(bmp[2..6].try_into().unwrap()) as usize,
            bmp.len()
        );
        assert_eq!(bmp.len(), 54 + 8 * 8 * 3);
    }

    #[test]
    fn seelen_owned_paths() {
        assert!(is_seelen_owned(
            r"C:\Users\x\AppData\Roaming\com.seelen.seelen-ui\wallpapers\a\thumbnail.jpg"
        ));
        assert!(!is_seelen_owned(
            r"C:\Windows\Web\Wallpaper\Windows\img0.jpg"
        ));
    }
}
