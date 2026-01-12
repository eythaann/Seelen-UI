use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{GetDC, GetPixel, ReleaseDC, CLR_INVALID, HDC},
};

use seelen_core::system_state::Color;

// Copy or Clone can't be implemented cuz this implements auto Drop/Release of the HDC
#[derive(Debug, PartialEq, Eq)]
pub struct DeviceContext {
    owner_hwnd: Option<HWND>,
    handle: HDC,
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe { ReleaseDC(self.owner_hwnd, self.handle) };
    }
}

impl DeviceContext {
    pub fn create(window: Option<HWND>) -> DeviceContext {
        DeviceContext {
            owner_hwnd: window,
            handle: unsafe { GetDC(window) },
        }
    }

    pub fn get_pixel(&self, x: i32, y: i32) -> Color {
        // the color is on the form 0xAABBGGRR, little endian of RGBA
        let color = unsafe { GetPixel(self.handle, x, y) }.0;
        if color == CLR_INVALID {
            return Color::default();
        }
        let [r, g, b, _] = color.to_le_bytes();
        Color::new(r, g, b, 0xFF)
    }
}
