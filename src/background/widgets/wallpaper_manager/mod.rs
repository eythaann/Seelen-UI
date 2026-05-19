pub mod cli;
pub mod handlers;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::{InvalidateRect, UpdateWindow},
    UI::WindowsAndMessaging::{
        FindWindowA, FindWindowExA, PostMessageW, SetParent, SetWindowLongPtrW, GWL_EXSTYLE,
        GWL_STYLE, WS_CHILDWINDOW, WS_CLIPSIBLINGS, WS_EX_ACCEPTFILES, WS_EX_APPWINDOW,
        WS_EX_WINDOWEDGE,
    },
};

use crate::{
    error::Result,
    pcstr,
    windows_api::{WindowEnumerator, WindowsApi},
};

pub struct SeelenWall {}

impl SeelenWall {
    fn setup_desktop_layer() -> Result<()> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };
        unsafe { PostMessageW(Some(progman), 0x052C, WPARAM(0xD), LPARAM(0x1))? };
        Ok(())
    }

    fn detect_worker_w() -> Result<Option<HWND>> {
        let mut worker_w = None;

        // CASE 1: Standard Windows 10/11 layout
        // WorkerW exists as a sibling to another WorkerW that contains SHELLDLL_DefView
        // 0x00010190 "" WorkerW
        //   0x000100EE "" SHELLDLL_DefView
        //     0x000100F0 "FolderView" SysListView32
        // 0x00100B8A "" WorkerW       <-- This is the WorkerW we want
        // 0x000100EC "Program Manager" Progman
        WindowEnumerator::new().for_each(|current| unsafe {
            if FindWindowExA(Some(current.hwnd()), None, pcstr!("SHELLDLL_DefView"), None).is_ok() {
                if let Ok(w) = FindWindowExA(None, Some(current.hwnd()), pcstr!("WorkerW"), None) {
                    worker_w = Some(w);
                }
            }
        })?;

        // CASE 2: Raised Desktop (Windows 11 with layered shell view)
        // Progman contains SHELLDLL_DefView and WorkerW as direct children
        // 0x000100EC "Program Manager" Progman
        //   0x000100EE "" SHELLDLL_DefView
        //   0x00100B8A "" WorkerW       <-- This is the WorkerW we want
        if worker_w.is_none() {
            let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };
            let has_defview =
                unsafe { FindWindowExA(Some(progman), None, pcstr!("SHELLDLL_DefView"), None) }
                    .is_ok();

            if has_defview {
                let mut attempts = 0;
                worker_w =
                    unsafe { FindWindowExA(Some(progman), None, pcstr!("WorkerW"), None).ok() };
                while worker_w.is_none() && attempts < 10 {
                    attempts += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    worker_w =
                        unsafe { FindWindowExA(Some(progman), None, pcstr!("WorkerW"), None).ok() };
                }
            }
        }

        Ok(worker_w)
    }

    pub fn try_set_under_desktop_items(hwnd: HWND) -> Result<()> {
        Self::setup_desktop_layer()?;
        let worker_w = Self::detect_worker_w()?;
        log::trace!("Setting under desktop, worker_w: {worker_w:?}");

        let worker_w = worker_w.ok_or("WorkerW not found")?;

        unsafe {
            // tao adds WS_EX_WINDOWEDGE to all windows; See tao#1153.
            let mut style = WindowsApi::get_styles(hwnd);
            style |= WS_CHILDWINDOW;
            style &= !WS_CLIPSIBLINGS;
            SetWindowLongPtrW(hwnd, GWL_STYLE, style.0 as isize);

            let mut ex_style = WindowsApi::get_ex_styles(hwnd);
            ex_style &= !WS_EX_ACCEPTFILES;
            ex_style &= !WS_EX_APPWINDOW;
            ex_style &= !WS_EX_WINDOWEDGE;
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style.0 as isize);

            SetParent(hwnd, Some(worker_w))?;
        }

        Ok(())
    }

    pub fn refresh_desktop() -> Result<()> {
        unsafe {
            let progman = FindWindowA(pcstr!("Progman"), None)?;
            if let Ok(shell_view) =
                FindWindowExA(Some(progman), None, pcstr!("SHELLDLL_DefView"), None)
            {
                InvalidateRect(Some(shell_view), None, true).ok()?;
                UpdateWindow(shell_view).ok()?;
            }
        }
        WindowsApi::refresh_desktop()
    }
}
