#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m128i, _mm_loadu_si128, _mm_setr_epi8, _mm_shuffle_epi8, _mm_storeu_si128,
};

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::{uint8x16_t, vld1q_u8, vqtbl1q_u8, vst1q_u8};

use image::{ImageBuffer, RgbaImage};
use windows::Win32::{
    Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
        GetDIBits, GetObjectW, HBITMAP, SelectObject,
    },
    UI::WindowsAndMessaging::{GetIconInfoExW, HICON, ICONINFOEXW},
};
use windows_core::Owned;

use crate::error::Result;

pub fn convert_hicon_to_rgba_image(hicon: &HICON) -> Result<RgbaImage> {
    unsafe {
        let mut icon_info = ICONINFOEXW {
            cbSize: std::mem::size_of::<ICONINFOEXW>() as u32,
            ..Default::default()
        };

        if !GetIconInfoExW(*hicon, &mut icon_info).as_bool() {
            return Err("Failed to get icon info".into());
        }

        let _hbm_mask = Owned::new(icon_info.hbmMask);
        let _hbm_color = Owned::new(icon_info.hbmColor);
        convert_hbitmap_to_rgba_image(icon_info.hbmColor)
    }
}

pub fn convert_hbitmap_to_rgba_image(hbitmap: HBITMAP) -> Result<RgbaImage> {
    unsafe {
        let mut bitmap = BITMAP::default();
        if GetObjectW(
            hbitmap.into(),
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
        let hbm_old = SelectObject(hdc_mem, hbitmap.into());

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

        let mut buffer = vec![0u8; (width * height * 4) as usize];

        let dibits_result = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        );

        // Cleanup GDI always, even on error
        SelectObject(hdc_mem, hbm_old);
        let _ = DeleteDC(hdc_mem);
        let _ = DeleteDC(hdc_screen);

        if dibits_result == 0 {
            return Err("Failed to get dibits".into());
        }

        // Alpha fix (important!)
        fix_alpha_channel(buffer.as_mut_slice());
        bgra_to_rgba(buffer.as_mut_slice());

        let image = ImageBuffer::from_raw(width as u32, height as u32, buffer)
            .ok_or("Failed to create image buffer")?;
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

/// Convert BGRA to RGBA
///
/// Uses SIMD to go fast
#[cfg(target_arch = "x86_64")]
fn bgra_to_rgba(data: &mut [u8]) {
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
fn bgra_to_rgba(data: &mut [u8]) {
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
