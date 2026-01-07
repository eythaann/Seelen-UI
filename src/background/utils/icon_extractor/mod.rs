mod queue;
use image::{GenericImageView, ImageBuffer, RgbaImage};
use itertools::Itertools;
use queue::{IconExtractor, IconExtractorRequest};
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{GetObjectW, BITMAP, BI_RGB};
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

use seelen_core::state::Icon;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m128i, _mm_loadu_si128, _mm_setr_epi8, _mm_shuffle_epi8, _mm_storeu_si128,
};

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::{uint8x16_t, vld1q_u8, vqtbl1q_u8, vst1q_u8};

use std::io::BufRead;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::modules::apps::application::msix::MsixAppsManager;
use crate::modules::start::application::START_MENU_MANAGER;
use crate::resources::RESOURCES;
use crate::utils::constants::SEELEN_COMMON;
use crate::utils::date_based_hex_id;
use crate::windows_api::types::AppUserModelId;
use crate::windows_api::WindowsApi;

/// Convert BGRA to RGBA
///
/// Uses SIMD to go fast
#[cfg(target_arch = "x86_64")]
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

// Uses NEON intrinsics to go fast
#[cfg(target_arch = "aarch64")]
pub fn bgra_to_rgba(data: &mut [u8]) {
    // The shuffle mask for converting BGRA -> RGBA
    let maskplain: [u8; 16] = [
        2, 1, 0, 3, // First pixel
        6, 5, 4, 7, // Second pixel
        10, 9, 8, 11, // Third pixel
        14, 13, 12, 15, // Fourth pixel
    ];
    // The shuffle mask for the conversion in NEON intrinsics
    let mask: uint8x16_t = unsafe { vld1q_u8(maskplain.as_ptr()) };
    // For each 16-byte chunk in your data
    for chunk in data.chunks_exact_mut(16) {
        let mut vector: uint8x16_t = unsafe { vld1q_u8(chunk.as_ptr()) };
        vector = unsafe { vqtbl1q_u8(vector, mask) };
        unsafe { vst1q_u8(chunk.as_mut_ptr(), vector) };
    }
}

pub fn convert_hicon_to_rgba_image(hicon: &HICON) -> Result<RgbaImage> {
    unsafe {
        let mut icon_info = ICONINFOEXW {
            cbSize: std::mem::size_of::<ICONINFOEXW>() as u32,
            ..Default::default()
        };

        if !GetIconInfoExW(*hicon, &mut icon_info).as_bool() {
            return Err("Failed to get icon info".into());
        }

        let mut bitmap = BITMAP::default();
        if GetObjectW(
            icon_info.hbmColor.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bitmap as *mut _ as *mut _),
        ) == 0
        {
            return Err("Failed to get bitmap info".into());
        }

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight.abs();

        let hdc_screen = CreateCompatibleDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbm_old = SelectObject(hdc_mem, icon_info.hbmColor.into());

        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -(height), // Negative for top-down bitmap
                biPlanes: 1,
                biBitCount: 32, // 4 bytes per pixel
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

        if GetDIBits(
            hdc_mem,
            icon_info.hbmColor,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        ) == 0
        {
            return Err("Failed to get dibits".into());
        }

        // Clean up
        SelectObject(hdc_mem, hbm_old);
        DeleteDC(hdc_mem).ok()?;
        DeleteDC(hdc_screen).ok()?;
        DeleteObject(icon_info.hbmColor.into()).ok()?;
        DeleteObject(icon_info.hbmMask.into()).ok()?;

        if bmp_info.bmiHeader.biBitCount != 32 {
            return Err("Icon is not 32 bit".into());
        }

        fix_alpha_channel(buffer.as_mut_slice());
        bgra_to_rgba(buffer.as_mut_slice());

        let image = ImageBuffer::from_raw(width as u32, height as u32, buffer)
            .expect("Failed to create image buffer");
        Ok(image)
    }
}

fn fix_alpha_channel(buffer: &mut [u8]) {
    let pixels = buffer.len() / 4;
    let mut has_non_zero_alpha = false;
    let mut has_varying_alpha = false;

    // Verify the alpha channel
    for i in 0..pixels {
        let alpha = buffer[i * 4 + 3];
        if alpha > 0 {
            has_non_zero_alpha = true;
        }
        if alpha > 0 && alpha < 255 {
            has_varying_alpha = true;
            break;
        }
    }

    // if all alpha values are 0, set them to 255
    if !has_non_zero_alpha {
        for i in 0..pixels {
            buffer[i * 4 + 3] = 255;
        }
    }
    // if there is premultiplied alpha
    else if has_varying_alpha {
        for i in 0..pixels {
            let alpha = buffer[i * 4 + 3];
            if alpha > 0 && alpha < 255 {
                let alpha_f = alpha as f32 / 255.0;
                let b = ((buffer[i * 4] as f32 / alpha_f).min(255.0)) as u8;
                let g = ((buffer[i * 4 + 1] as f32 / alpha_f).min(255.0)) as u8;
                let r = ((buffer[i * 4 + 2] as f32 / alpha_f).min(255.0)) as u8;
                buffer[i * 4] = b;
                buffer[i * 4 + 1] = g;
                buffer[i * 4 + 2] = r;
            }
        }
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
        let normalized = path
            .canonicalize()?
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_owned();
        let path_str = normalized.encode_utf16().chain(Some(0)).collect_vec();

        let mut file_info = SHFILEINFOW::default();
        let result = SHGetFileInfoW(
            PCWSTR(path_str.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_SYSICONINDEX,
        );

        if result == 0 {
            return Err("Failed to get file information".into());
        }

        // file_info.iIcon = 0 is a valid icon but it is the default icon for files on Windows
        // so we will handle this as no icon to avoid generate unnecessary artifacts
        if file_info.iIcon == 0 {
            return Err("Icon index is 0".into());
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

const SQUARE_MARGIN: f32 = 0.1;
const ASPECT_TOLERANCE: f32 = 0.05;
const OPACITY_THRESHOLD: u8 = 254;

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

// maintain this function as documentation for url files
#[allow(dead_code)]
fn get_icon_from_url_file(path: &Path) -> Result<RgbaImage> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);

    let mut path = None;
    // in theory .url files are encoded in UTF-8 so we don't need to use OsString
    for line in reader.lines() {
        if let Some(stripped) = line?.strip_prefix("IconFile=") {
            path = Some(PathBuf::from(stripped));
            break;
        }
    }

    let path = match path {
        Some(icon_file) => icon_file,
        None => return Err("Failed to get icon".into()),
    };

    get_icon_from_file(&path)
}

pub fn extract_and_save_icon_from_file<T: AsRef<Path>>(path: T) {
    IconExtractor::request(IconExtractorRequest::Path(path.as_ref().to_path_buf()));
}

/// returns the path of the icon extracted from the executable or copied if is an UWP app.
///
/// If the icon already exists, it returns the path instead overriding, this is needed for allow user custom icons.
///
/// umid on this case only applys to Property Store umid
pub fn _extract_and_save_icon_from_file(origin: &Path, umid: Option<String>) -> Result<()> {
    if !origin.exists() || origin.is_dir() {
        return Err(format!("File not found: {}", origin.display()).into());
    }

    let origin_ext = match origin.extension() {
        Some(ext) => ext.to_string_lossy().to_lowercase(),
        // no extension === no icon
        None => return Ok(()),
    };

    // ico files are by itself an icon
    if origin_ext == "ico" {
        return Ok(());
    }

    let is_exe_file = origin_ext == "exe";
    let is_lnk_file = origin_ext == "lnk";
    let is_url_file = origin_ext == "url";

    if is_exe_file || is_lnk_file || is_url_file {
        if RESOURCES.has_app_icon(None, Some(origin)) {
            return Ok(());
        }
    } else if RESOURCES.has_shared_file_icon(origin) {
        return Ok(());
    }

    let file_name = origin.file_name().ok_or("Failed to get file name")?;
    let filestem = origin.file_stem().ok_or("Failed to get file stem")?;

    let gen_icon_filename = format!("{}_{}.png", filestem.to_string_lossy(), date_based_hex_id());
    let mut gen_icon = Icon {
        base: Some(gen_icon_filename.clone()),
        ..Default::default()
    };

    log::trace!("Extracting icon for {file_name:?}");

    if origin_ext == "url" {
        if let Ok(icon) = get_icon_from_url_file(origin) {
            gen_icon.is_aproximately_square = is_aproximately_a_square(&icon);
            icon.save(
                SEELEN_COMMON
                    .system_icon_pack_path()
                    .join(&gen_icon_filename),
            )?;
            RESOURCES.add_system_app_icon(None, Some(origin), gen_icon);
        }
        return Ok(());
    }

    if is_lnk_file {
        let lnk_icon_path = match WindowsApi::resolve_lnk_custom_icon_path(origin) {
            Ok(icon_path) => icon_path,
            Err(_) => {
                let (target, _) = WindowsApi::resolve_lnk_target(origin)?;
                target
            }
        };

        if lnk_icon_path
            .extension()
            .is_some_and(|ext| ext.to_string_lossy().to_lowercase() != "ico")
        {
            _extract_and_save_icon_from_file(&lnk_icon_path, umid.clone())?;
            RESOURCES.add_system_icon_redirect(umid, origin, &lnk_icon_path);
            return Ok(());
        }
    }

    // try get the icon directly from the file
    let icon = match get_icon_from_file(origin) {
        Ok(icon) => icon,
        Err(_) => {
            log::trace!("Icon not found for {}", origin.display());
            return Ok(());
        }
    };

    gen_icon.is_aproximately_square = is_aproximately_a_square(&icon);

    if is_exe_file || is_lnk_file {
        icon.save(
            SEELEN_COMMON
                .system_icon_pack_path()
                .join(&gen_icon_filename),
        )?;
        RESOURCES.add_system_app_icon(umid.as_deref(), Some(origin), gen_icon);
    } else {
        let gen_icon_filename = format!("{}_{}.png", origin_ext, date_based_hex_id());
        icon.save(
            SEELEN_COMMON
                .system_icon_pack_path()
                .join(&gen_icon_filename),
        )?;
        gen_icon.base = Some(gen_icon_filename);
        RESOURCES.add_system_file_icon(&origin_ext, gen_icon);
    }

    Ok(())
}

pub fn extract_and_save_icon_umid(aumid: &AppUserModelId) {
    IconExtractor::request(IconExtractorRequest::AppUMID(aumid.clone()));
}

/// returns the path of the icon extracted from the app with the specified package app user model id.
pub fn _extract_and_save_icon_umid(aumid: &AppUserModelId) -> Result<()> {
    match aumid {
        AppUserModelId::Appx(app_umid) => {
            let msix_manager = MsixAppsManager::instance();
            let path = msix_manager.get_app_path(app_umid)?;
            {
                if RESOURCES.has_app_icon(Some(aumid.as_str()), path.as_deref()) {
                    return Ok(());
                }
            }

            log::trace!("Extracting icon for {app_umid:?}");
            let mut gen_icon = Icon::default();
            let (light_path, dark_path) = msix_manager.get_app_icon_path(app_umid)?;

            let name = date_based_hex_id();

            let light_rgba = image::open(&light_path)?.to_rgba8();
            let light_rgba = crop_transparent_borders(&light_rgba);

            if light_path != dark_path {
                let dark_rgba = image::open(&dark_path)?.to_rgba8();
                let dark_rgba = crop_transparent_borders(&dark_rgba);

                light_rgba.save(
                    SEELEN_COMMON
                        .system_icon_pack_path()
                        .join(format!("{name}_light.png")),
                )?;
                dark_rgba.save(
                    SEELEN_COMMON
                        .system_icon_pack_path()
                        .join(format!("{name}_dark.png")),
                )?;

                gen_icon.light = Some(format!("{name}_light.png"));
                gen_icon.dark = Some(format!("{name}_dark.png"));
            } else {
                light_rgba.save(
                    SEELEN_COMMON
                        .system_icon_pack_path()
                        .join(format!("{name}.png")),
                )?;
                gen_icon.base = Some(format!("{name}.png"));
            }

            gen_icon.is_aproximately_square = is_aproximately_a_square(&light_rgba);

            RESOURCES.add_system_app_icon(Some(app_umid), path.as_deref(), gen_icon);
            Ok(())
        }
        AppUserModelId::PropertyStore(app_umid) => {
            let start = START_MENU_MANAGER.load();
            let lnk = start
                .get_by_file_umid(app_umid)
                .ok_or(format!("No shortcut found for umid {app_umid}"))?;

            {
                if RESOURCES.has_app_icon(Some(aumid.as_str()), Some(&lnk.path)) {
                    return Ok(());
                }
            }

            _extract_and_save_icon_from_file(&lnk.path, Some(app_umid.clone()))?;
            Ok(())
        }
    }
}
