use std::{os::windows::io::AsRawHandle, thread::JoinHandle};

use windows::Win32::{
    Foundation::{HANDLE, HWND, LPARAM, POINT, WPARAM},
    System::Threading::GetThreadId,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, FindWindowExW, FindWindowW, GetCursorPos, GetMessageW,
        PostThreadMessageW, RegisterClassW, TranslateMessage, CS_HREDRAW, CS_VREDRAW,
        CW_USEDEFAULT, MSG, WM_QUIT, WNDCLASSW, WNDPROC, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST, WS_OVERLAPPEDWINDOW,
    },
};
use windows_core::{w, PCWSTR};

use crate::windows_api::WindowsApi;

pub type WindowProcedure = WNDPROC;

pub struct Util;
impl Util {
    /// Creates a hidden message window.
    ///
    /// Returns a handle to the created window.
    pub fn create_message_window(
        class_name: &str,
        window_procedure: WindowProcedure,
    ) -> crate::Result<isize> {
        let class_name = Self::to_wide(class_name);

        let class = WNDCLASSW {
            lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: window_procedure,
            ..Default::default()
        };

        let class_atom = unsafe { RegisterClassW(&class) };

        if class_atom == 0 {
            return Err("Failed to register window class".into());
        }

        let handle = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_APPWINDOW | WS_EX_TOPMOST,
                PCWSTR::from_raw(class_name.as_ptr()),
                PCWSTR::from_raw(class_name.as_ptr()),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                Some(class.hInstance),
                None,
            )
        }?;

        Ok(handle.0 as isize)
    }

    /// Starts a message loop on the current thread.
    ///
    /// This function will block until the message loop is killed. Use
    /// `Util::kill_message_loop` to terminate the message loop.
    pub fn run_message_loop() {
        let mut msg = MSG::default();

        loop {
            if unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                let _ = unsafe { TranslateMessage(&msg) };
                unsafe { DispatchMessageW(&msg) };
            } else {
                break;
            }
        }
    }

    /// Gracefully terminates the message loop on the given thread.
    pub fn kill_message_loop<T>(thread: &JoinHandle<T>) -> crate::Result<()> {
        let handle = thread.as_raw_handle();
        let handle = HANDLE(handle);
        let thread_id = unsafe { GetThreadId(handle) };

        unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM::default(), LPARAM::default()) }?;

        Ok(())
    }

    /// Converts a string to a wide string.
    pub fn to_wide(string: &str) -> Vec<u16> {
        string.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Packs two 16-bit values into a 32-bit value. This is commonly used
    /// for `WPARAM` and `LPARAM` values.
    ///
    /// Equivalent to the Win32 `MAKELPARAM` and `MAKEWPARAM` macros.
    pub fn pack_i32(low: i16, high: i16) -> i32 {
        low as i32 | ((high as i32) << 16)
    }

    /// Gets the mouse position in screen coordinates.
    pub fn cursor_position() -> crate::Result<(i32, i32)> {
        let mut point = POINT { x: 0, y: 0 };
        unsafe { GetCursorPos(&mut point) }?;
        Ok((point.x, point.y))
    }

    /// Finds the Windows tray window, ignoring a specific window handle.
    pub fn find_tray_window(hwnd_ignore: isize) -> Option<isize> {
        let mut taskbar_hwnd = unsafe { FindWindowW(w!("Shell_TrayWnd"), None) }.ok()?;

        if hwnd_ignore != 0 {
            while taskbar_hwnd == HWND(hwnd_ignore as _) {
                taskbar_hwnd = WindowsApi::find_window(
                    None,
                    Some(taskbar_hwnd),
                    None,
                    Some("Shell_TrayWnd".to_owned()),
                )
                .ok()?;
            }
        }

        Some(taskbar_hwnd.0 as isize)
    }

    /// Finds the toolbar window (contains tray icons) within the given tray
    /// window.
    pub fn find_tray_toolbar_window(tray_handle: isize) -> Option<isize> {
        let notify = WindowsApi::find_window(
            None,
            Some(HWND(tray_handle as _)),
            None,
            Some("TrayNotifyWnd".to_owned()),
        )
        .ok()?;

        log::info!("Found TrayNotifyWnd: {:?}", notify);

        let pager =
            WindowsApi::find_window(Some(notify), None, None, Some("SysPager".to_owned())).ok()?;
        log::info!("Found SysPager: {:?}", pager);

        let toolbar =
            WindowsApi::find_window(Some(pager), None, None, Some("ToolbarWindow32".to_owned()))
                .ok()?;
        log::info!("Found ToolbarWindow32: {:?}", toolbar);

        Some(toolbar.0 as isize)
    }

    /// Finds the toolbar window (contains tray icons) for overflowed icons.
    /// This is the window accessed via the chevron button in the Windows
    /// taskbar.
    pub fn find_overflow_toolbar_window() -> Option<isize> {
        let notify = unsafe { FindWindowW(w!("NotifyIconOverflowWindow"), None) }.ok()?;

        let toolbar =
            unsafe { FindWindowExW(Some(notify), None, w!("ToolbarWindow32"), None) }.ok()?;

        Some(toolbar.0 as isize)
    }
}
