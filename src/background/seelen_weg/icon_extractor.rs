use color_eyre::eyre::eyre;
use image::{GenericImageView, ImageBuffer, RgbaImage};
use itertools::Itertools;
use tauri::{AppHandle, Manager};
use windows::core::PCWSTR;
use windows::Win32::{
    Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, DIB_RGB_COLORS,
    },
    Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES,
    UI::{
        Controls::{IImageList, ILD_TRANSPARENT},
        Shell::{SHGetFileInfoW, SHGetImageList, SHFILEINFOW, SHGFI_SYSICONINDEX, SHIL_JUMBO},
        WindowsAndMessaging::{DestroyIcon, GetIconInfoExW, HICON, ICONINFOEXW},
    },
};

use std::arch::x86_64::{
    __m128i, _mm_loadu_si128, _mm_setr_epi8, _mm_shuffle_epi8, _mm_storeu_si128,
};
use std::ffi::OsStr;
use std::io::BufRead;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::error_handler::Result;
use crate::modules::uwp::UWP_MANAGER;
use crate::seelen::get_app_handle;
use crate::trace_lock;
use crate::windows_api::WindowsApi;

/// Convert BGRA to RGBA
///
/// Uses SIMD to go fast
pub fn bgra_to_rgba(data: &mut [u8]) {
    // The shuffle mask for converting BGRA -> RGBA
    let mask: __m128i = unsafe {
        _mm_setr_epi8(
            2, 1, 0, 3, // First pixel
            6, 5, 4, 7, // Second pixel
            10, 9, 8, 11, // Third pixel
            14, 13, 12, 15, // Fourth pixel
        )
    };
    // For each 16-byte chunk in your data
    for chunk in data.chunks_exact_mut(16) {
        let mut vector = unsafe { _mm_loadu_si128(chunk.as_ptr() as *const __m128i) };
        vector = unsafe { _mm_shuffle_epi8(vector, mask) };
        unsafe { _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, vector) };
    }
}

pub fn convert_hicon_to_rgba_image(hicon: &HICON) -> Result<RgbaImage> {
    unsafe {
        let mut icon_info = ICONINFOEXW {
            cbSize: std::mem::size_of::<ICONINFOEXW>() as u32,
            ..Default::default()
        };

        if !GetIconInfoExW(*hicon, &mut icon_info).as_bool() {
            return Err(eyre!("Failed to get icon info").into());
        }
        let hdc_screen = CreateCompatibleDC(None);
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        let hbm_old = SelectObject(hdc_mem, icon_info.hbmColor);

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: icon_info.xHotspot as i32 * 2,
                biHeight: -(icon_info.yHotspot as i32 * 2),
                biPlanes: 1,
                biBitCount: 32, // 4 bytes per pixel
                biCompression: DIB_RGB_COLORS.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buffer: Vec<u8> =
            vec![0; (icon_info.xHotspot * 2 * icon_info.yHotspot * 2 * 4) as usize];

        if GetDIBits(
            hdc_mem,
            icon_info.hbmColor,
            0,
            icon_info.yHotspot * 2,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        ) == 0
        {
            return Err(eyre!("Failed to get dibits").into());
        }

        // Clean up
        SelectObject(hdc_mem, hbm_old);
        DeleteDC(hdc_mem).ok()?;
        DeleteDC(hdc_screen).ok()?;
        DeleteObject(icon_info.hbmColor).ok()?;
        DeleteObject(icon_info.hbmMask).ok()?;

        if bmp_info.bmiHeader.biBitCount != 32 {
            return Err("Icon is not 32 bit".into());
        }

        bgra_to_rgba(buffer.as_mut_slice());

        let image = ImageBuffer::from_raw(icon_info.xHotspot * 2, icon_info.yHotspot * 2, buffer)
            .expect("Failed to create image buffer");
        Ok(image)
    }
}

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

pub fn get_icon_from_file(path: &Path) -> Result<RgbaImage> {
    unsafe {
        let path_str = path.as_os_str().encode_wide().chain(Some(0)).collect_vec();

        let mut file_info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(path_str.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_SYSICONINDEX,
        );

        // file_info.iIcon = 0 is a valid icon but it is the default icon for files on Windows
        // so we will handle this as no icon to avoid generate unnecessary artifacts
        if result == 0 || file_info.iIcon == 0 {
            return Err(eyre!("Failed to get icon").into());
        }

        let image_list: IImageList = SHGetImageList(SHIL_JUMBO as i32)?;
        // if 256x256 icon is not available, will use the icons with the most color depth and size
        // this is useful for some icons where color depth is less than 32,
        // example: icon of 124x124 16bits and other 64x64 32bits this will return the 32bits icon
        // color depth is prioritized over size
        let icon = image_list.GetIcon(file_info.iIcon, ILD_TRANSPARENT.0)?;
        let image = crop_transparent_borders(&convert_hicon_to_rgba_image(&icon)?);
        DestroyIcon(icon)?;
        Ok(image)
    }
}

pub fn get_icon_from_url_file(path: &Path) -> Result<RgbaImage> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);

    let mut path = None;
    // in theory .url files are encoded in UTF-8 so we don't need to use OsString
    for line in reader.lines().map_while(Result::ok) {
        if let Some(stripped) = line.strip_prefix("IconFile=") {
            path = Some(PathBuf::from(stripped));
            break;
        }
    }

    let path = match path {
        Some(icon_file) => icon_file,
        None => return Err(eyre!("Failed to get icon").into()),
    };

    get_icon_from_file(&path)
}

/// returns the path of the icon extracted from the executable or copied if is an UWP app.
///
/// If the icon already exists, it returns the path instead overriding, this is needed for allow user custom icons.
pub fn extract_and_save_icon(_handle: &AppHandle, file_path: &str) -> Result<PathBuf> {
    extract_and_save_icon_v2(PathBuf::from(file_path))
}

/// returns the path of the icon extracted from the executable or copied if is an UWP app.
///
/// If the icon already exists, it returns the path instead overriding, this is needed for allow user custom icons.
pub fn extract_and_save_icon_v2<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let path = path.as_ref();
    if path.is_dir() {
        return Err("Path is a directory".into());
    }

    let gen_icons_paths = get_app_handle().path().app_data_dir()?.join("icons");
    if !gen_icons_paths.exists() {
        std::fs::create_dir_all(&gen_icons_paths)?;
    }

    let file_name = path.file_name().ok_or("Failed to get file name")?;
    let path_to_save = gen_icons_paths.join(file_name).with_extension("png");
    if path_to_save.exists() {
        return Ok(path_to_save);
    }

    let filename = file_name.to_string_lossy().to_string();
    log::trace!("Extracting icon for \"{}\"", filename);

    let ext = path.extension();

    // try get icons for URLs
    if ext == Some(OsStr::new("url")) {
        let icon = get_icon_from_url_file(path)?;
        icon.save(&path_to_save)?;
        return Ok(path_to_save);
    }

    // try get icons for UWP/Msix apps
    if ext == Some(OsStr::new("exe")) {
        if let Some(package) = trace_lock!(UWP_MANAGER).get_from_path(path) {
            if let Some(uwp_icon_path) = package.get_light_icon(&filename) {
                log::debug!("Copying UWP icon from \"{}\"", uwp_icon_path.display());
                std::fs::copy(uwp_icon_path, &path_to_save)?;
                return Ok(path_to_save);
            }
        }
    }

    // try get the icon directly from the file
    if let Ok(icon) = get_icon_from_file(path) {
        icon.save(&path_to_save)?;
        return Ok(path_to_save);
    }

    // if the lnk don't have an icon, try to extract it from the target
    if ext == Some(OsStr::new("lnk")) {
        let target = WindowsApi::resolve_lnk_target(path)?;
        return extract_and_save_icon_v2(&target);
    }

    Err("Failed to extract icon".into())
}
