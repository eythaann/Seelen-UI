use color_eyre::eyre::eyre;
use image::ImageBuffer;
use image::RgbaImage;
use itertools::Itertools;
use tauri::AppHandle;
use widestring::U16CString;
use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
use windows::Win32::Graphics::Gdi::DeleteDC;
use windows::Win32::Graphics::Gdi::DeleteObject;
use windows::Win32::Graphics::Gdi::GetDIBits;
use windows::Win32::Graphics::Gdi::SelectObject;
use windows::Win32::Graphics::Gdi::BITMAPINFO;
use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::DestroyIcon;
use windows::Win32::UI::WindowsAndMessaging::GetIconInfoExW;
use windows::Win32::UI::WindowsAndMessaging::HICON;
use windows::Win32::UI::WindowsAndMessaging::ICONINFOEXW;

use std::arch::x86_64::__m128i;
use std::arch::x86_64::_mm_loadu_si128;
use std::arch::x86_64::_mm_setr_epi8;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_shuffle_epi8;
use std::arch::x86_64::_mm_storeu_si128;
use std::path::PathBuf;

use crate::error_handler::Result;
use crate::utils::app_data_path;
use crate::utils::filename_from_path;

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

pub fn get_images_from_exe(executable_path: &str) -> Result<Vec<RgbaImage>> {
    unsafe {
        let path_cstr = U16CString::from_str(executable_path).map_err(|_| eyre!("Invalid path"))?;
        let path_pcwstr = PCWSTR(path_cstr.as_ptr());
        let num_icons_total = ExtractIconExW(path_pcwstr, -1, None, None, 0);
        if num_icons_total == 0 {
            return Ok(Vec::new()); // No icons extracted
        }

        let mut large_icons = vec![HICON::default(); num_icons_total as usize];
        let mut small_icons = vec![HICON::default(); num_icons_total as usize];
        let num_icons_fetched = ExtractIconExW(
            path_pcwstr,
            0,
            Some(large_icons.as_mut_ptr()),
            Some(small_icons.as_mut_ptr()),
            num_icons_total,
        );

        if num_icons_fetched == 0 {
            return Ok(Vec::new()); // No icons extracted
        }

        let images = large_icons
            .iter()
            .chain(small_icons.iter())
            .map(convert_hicon_to_rgba_image)
            .filter_map(|r| match r {
                Ok(img) => Some(img),
                Err(e) => {
                    log::error!("Failed to convert HICON to RgbaImage: {:?}", e);
                    None
                }
            })
            .collect_vec();

        large_icons
            .iter()
            .chain(small_icons.iter())
            .filter(|icon| !icon.is_invalid())
            .map(|icon| DestroyIcon(*icon))
            .filter_map(|r| r.err())
            .for_each(|e| {
                log::error!("Failed to destroy icon: {:?}", e);
            });

        Ok(images)
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
                biBitCount: 32,
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

        bgra_to_rgba(buffer.as_mut_slice());

        let image = ImageBuffer::from_raw(icon_info.xHotspot * 2, icon_info.yHotspot * 2, buffer)
            .expect("Failed to create image buffer");
        Ok(image)
    }
}

/// returns the path of the icon extracted from the executable or copied if is an UWP app.
///
/// If the icon already exists, it returns the path instead overriding, this is needed for allow user custom icons.
pub fn extract_and_save_icon(handle: &AppHandle, exe_path: &str) -> Result<PathBuf> {
    let gen_icons_paths = app_data_path(handle).join("icons");
    if !gen_icons_paths.exists() {
        std::fs::create_dir_all(&gen_icons_paths)?;
    }

    let filename = filename_from_path(exe_path);
    let icon_path = gen_icons_paths.join(filename.replace(".exe", ".png"));
    let icon_path_uwp = gen_icons_paths.join(filename.replace(".exe", "_uwp.png"));

    if icon_path_uwp.exists() {
        return Ok(icon_path_uwp);
    }

    if icon_path.exists() {
        return Ok(icon_path);
    }

    let images = get_images_from_exe(exe_path);
    if let Ok(images) = images {
        // icon on index 0 always is the app showed icon
        if let Some(icon) = images.first() {
            icon.save(&icon_path)?;
        }
    }

    Ok(icon_path)
}
