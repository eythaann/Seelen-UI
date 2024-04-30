use windows::Win32::{Foundation::{BOOL, HWND, LPARAM}, UI::WindowsAndMessaging::EnumWindows};

use crate::{error_handler::log_if_error, seelen::SEELEN};

use super::WindowManager;

impl WindowManager {
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut seelen = SEELEN.lock();
        if let Some(wm) = seelen.wm_mut() {
            if Self::is_manageable_window(hwnd, true) {
                log_if_error(wm.add_hwnd(hwnd));
            }
        }
        true.into()
    }

    pub fn enum_windows() {
        std::thread::spawn(|| unsafe {
            log::trace!("Enumerating windows");
            log_if_error(EnumWindows(Some(Self::enum_windows_proc), LPARAM(0)));
            log::trace!("Finished enumerating windows");
        });
    }
}
