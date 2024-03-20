use windows::Win32::{
    Foundation::HWND,
    System::Threading::{AttachThreadInput, GetCurrentProcessId, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::SetFocus,
        WindowsAndMessaging::{
            AllowSetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
        },
    },
};

use crate::error_handler::Result;

pub struct WindowsApi {}
impl WindowsApi {
    pub fn window_thread_process_id(hwnd: HWND) -> (u32, u32) {
        let mut process_id: u32 = 0;

        // Behaviour is undefined if an invalid HWND is given
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid
        let thread_id = unsafe {
            GetWindowThreadProcessId(hwnd, Option::from(std::ptr::addr_of_mut!(process_id)))
        };

        (process_id, thread_id)
    }

    pub fn current_thread_id() -> u32 {
        unsafe { GetCurrentThreadId() }
    }

    pub fn attach_thread_input(thread_id: u32, target_thread_id: u32, attach: bool) -> Result<()> {
        unsafe {
            AttachThreadInput(thread_id, target_thread_id, attach);
        }
        Ok(())
    }

    pub fn allow_set_foreground_window(process_id: u32) -> Result<()> {
        unsafe {
            AllowSetForegroundWindow(process_id)?;
        }
        Ok(())
    }

    pub fn set_foreground_window(hwnd: HWND) -> Result<()> {
        unsafe {
            SetForegroundWindow(hwnd);
        }
        Ok(())
    }

    pub fn set_focus(hwnd: HWND) -> Result<()> {
        unsafe { SetFocus(hwnd) };
        Ok(())
    }

    pub fn current_process_id() -> u32 {
        unsafe { GetCurrentProcessId() }
    }
}
