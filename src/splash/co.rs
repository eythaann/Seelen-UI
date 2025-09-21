use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{GetDesktopWindow, GetWindowRect},
};

pub fn get_desktop_rect() -> RECT {
    let hwnd: HWND = unsafe { GetDesktopWindow() };
    let mut rect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut rect).expect("Failed to get desktop window rect") };
    rect
}
