use windows::Win32::{
    Foundation::{BOOL, LPARAM, HWND},
    UI::WindowsAndMessaging::EnumWindows,
};

use crate::{error_handler::log_if_error, seelen::SEELEN};

use super::SeelenWeg;

impl SeelenWeg {
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut seelen = SEELEN.lock();
        if let Some(weg) = seelen.weg_mut() {
            if SeelenWeg::is_real_window(hwnd) {
                weg.add_hwnd(hwnd);
            }
        }
        true.into()
    }

    pub fn enum_opened_windows() {
        std::thread::spawn(|| unsafe {
            log_if_error(EnumWindows(Some(Self::enum_windows_proc), LPARAM(0)));
        });
    }
}
