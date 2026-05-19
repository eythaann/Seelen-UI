pub mod cli;
pub mod handlers;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::{InvalidateRect, UpdateWindow},
    UI::WindowsAndMessaging::{
        FindWindowA, FindWindowExA, GetParent, PostMessageW, SetParent, SetWindowLongPtrW,
        SetWindowPos, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOOWNERZORDER, SWP_NOSIZE, WS_CHILDWINDOW, WS_EX_ACCEPTFILES, WS_EX_APPWINDOW,
        WS_EX_NOREDIRECTIONBITMAP, WS_EX_WINDOWEDGE,
    },
};

use crate::{
    error::Result,
    pcstr,
    windows_api::{WindowEnumerator, WindowsApi},
};

struct DesktopInfo {
    progman: HWND,
    worker_w: Option<HWND>,
    defview: Option<HWND>,
    is_raised_desktop: bool,
}

pub struct SeelenWall {}

impl SeelenWall {
    fn setup_desktop_layer() -> Result<()> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };
        unsafe { PostMessageW(Some(progman), 0x052C, WPARAM(0xD), LPARAM(0x1))? };
        Ok(())
    }

    fn detect_desktop_info() -> Result<DesktopInfo> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };

        let mut worker_w = None;
        let mut defview = None;
        let mut is_raised_desktop = false;

        // CASE 1: Standard Windows 10/11 layout
        // WorkerW exists as a sibling to another WorkerW that contains SHELLDLL_DefView
        // 0x00010190 "" WorkerW
        //   0x000100EE "" SHELLDLL_DefView
        //     0x000100F0 "FolderView" SysListView32
        // 0x00100B8A "" WorkerW       <-- This is the WorkerW we want
        // 0x000100EC "Program Manager" Progman
        // We enumerate all Windows, until we find one, that has the SHELLDLL_DefView as a child.
        // If we found that window, we take its next sibling and assign it to workerw.
        WindowEnumerator::new().for_each(|current| unsafe {
            if FindWindowExA(Some(current.hwnd()), None, pcstr!("SHELLDLL_DefView"), None).is_ok() {
                if let Ok(w) = FindWindowExA(None, Some(current.hwnd()), pcstr!("WorkerW"), None) {
                    worker_w = Some(w);
                }
            }
        })?;

        // CASE 2: Raised Desktop (Windows 11 with layered shell view)
        // Progman contains SHELLDLL_DefView and WorkerW as children
        // 0x000100EC "Program Manager" Progman
        //   0x000100EE "" SHELLDLL_DefView
        //     0x000100F0 "FolderView" SysListView32
        //   0x00100B8A "" WorkerW       <-- This is the WorkerW we want
        if worker_w.is_none() {
            defview = unsafe {
                FindWindowExA(Some(progman), None, pcstr!("SHELLDLL_DefView"), None).ok()
            };

            if defview.is_some() {
                let ex_style = WindowsApi::get_ex_styles(progman);
                is_raised_desktop = ex_style.contains(WS_EX_NOREDIRECTIONBITMAP);

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

        Ok(DesktopInfo {
            progman,
            worker_w,
            defview,
            is_raised_desktop,
        })
    }

    fn attach_standard_desktop(hwnd: HWND, desktop_info: &DesktopInfo) -> Result<()> {
        let worker_w = desktop_info
            .worker_w
            .ok_or("WorkerW not found for standard desktop")?;

        unsafe {
            if GetParent(hwnd).ok() != Some(worker_w) {
                // Apply same style cleanup as raised desktop — tao adds WS_EX_WINDOWEDGE to
                // all windows; leaving it on causes the window to disappear after SetParent
                // to WorkerW on some configurations (MSIX in particular). See tao#1153.
                let mut style = WindowsApi::get_styles(hwnd);
                style |= WS_CHILDWINDOW;
                SetWindowLongPtrW(hwnd, GWL_STYLE, style.0 as isize);

                let mut ex_style = WindowsApi::get_ex_styles(hwnd);
                ex_style &= !WS_EX_ACCEPTFILES;
                ex_style &= !WS_EX_APPWINDOW;
                ex_style &= !WS_EX_WINDOWEDGE;
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style.0 as isize);

                SetParent(hwnd, Some(worker_w))?;
            }

            SetWindowPos(
                hwnd,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOOWNERZORDER,
            )?;
        }

        Ok(())
    }

    /// For raised desktop (Windows 11 layered shell), parent to Progman and position below DefView.
    // workaround for https://github.com/tauri-apps/tao/issues/1153 (WS_EX_WINDOWEDGE removal)
    fn attach_raised_desktop(hwnd: HWND, desktop_info: &DesktopInfo) -> Result<()> {
        let defview = desktop_info
            .defview
            .ok_or("DefView not found for raised desktop")?;

        unsafe {
            let current_parent = GetParent(hwnd).ok();
            if current_parent != Some(desktop_info.progman) {
                let mut style = WindowsApi::get_styles(hwnd);
                style |= WS_CHILDWINDOW;
                SetWindowLongPtrW(hwnd, GWL_STYLE, style.0 as isize);

                let mut ex_style = WindowsApi::get_ex_styles(hwnd);
                ex_style &= !WS_EX_ACCEPTFILES;
                ex_style &= !WS_EX_APPWINDOW;
                ex_style &= !WS_EX_WINDOWEDGE;
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style.0 as isize);

                SetParent(hwnd, Some(desktop_info.progman))?;
            }

            WindowsApi::set_position(
                hwnd,
                Some(defview),
                &Default::default(),
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )?;

            if let Some(worker_w) = desktop_info.worker_w {
                Self::ensure_workerw_z_order(worker_w)?;
            }
        }

        Ok(())
    }

    fn ensure_workerw_z_order(worker_w: HWND) -> Result<()> {
        unsafe {
            SetWindowPos(
                worker_w,
                Some(HWND_BOTTOM),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )?;
        }
        Ok(())
    }

    pub fn try_set_under_desktop_items(hwnd: HWND) -> Result<()> {
        Self::setup_desktop_layer()?;
        let desktop_info = Self::detect_desktop_info()?;
        if desktop_info.is_raised_desktop {
            Self::attach_raised_desktop(hwnd, &desktop_info)
        } else {
            Self::attach_standard_desktop(hwnd, &desktop_info)
        }
    }

    /// this is only needed on the case 2 of try_set_inside_workerw
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
