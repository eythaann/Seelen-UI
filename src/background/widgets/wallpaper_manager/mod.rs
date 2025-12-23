mod hook;

use base64::Engine;
use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::Gdi::{InvalidateRect, UpdateWindow},
    UI::WindowsAndMessaging::{
        FindWindowA, FindWindowExA, GetParent, PostMessageW, SetParent, SetWindowLongPtrW,
        SetWindowPos, GWL_EXSTYLE, GWL_STYLE, HWND_BOTTOM, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOSIZE, WS_CHILDWINDOW, WS_EX_ACCEPTFILES,
        WS_EX_APPWINDOW, WS_EX_NOREDIRECTIONBITMAP, WS_EX_WINDOWEDGE,
    },
};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error, pcstr,
    widgets::WebviewArgs,
    windows_api::{WindowEnumerator, WindowsApi},
};

struct DesktopInfo {
    progman: HWND,
    worker_w: Option<HWND>,
    defview: Option<HWND>,
    is_raised_desktop: bool,
}

pub struct SeelenWall {
    window: WebviewWindow,
}

impl Drop for SeelenWall {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        log_error!(self.window.destroy());
    }
}

impl SeelenWall {
    pub const TITLE: &str = "Seelen Wall";
    const TARGET: &str = "@seelen/wallpaper-manager";

    pub fn new() -> Result<Self> {
        log::info!("Creating {}", Self::TARGET);
        Ok(Self {
            window: Self::create_window()?,
        })
    }

    fn create_window() -> Result<WebviewWindow> {
        let handle = get_app_handle();
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Self::TARGET);
        let args = WebviewArgs::new();

        let builder = tauri::WebviewWindowBuilder::new(
            handle,
            label,
            tauri::WebviewUrl::App("react/wallpaper_manager/index.html".into()),
        )
        .title(Self::TITLE)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .resizable(false)
        .decorations(false)
        .shadow(false)
        .transparent(true)
        .focusable(false)
        .visible(false)
        .skip_taskbar(true)
        .drag_and_drop(false)
        .disable_drag_drop_handler()
        .always_on_bottom(true)
        .data_directory(args.data_directory())
        .additional_browser_args(&args.to_string());

        // workaround for https://github.com/tauri-apps/tao/issues/1153
        /* if let Ok(_progman) = WindowsApi::find_window(None, None, None, Some("Progman")) {
            let mut progman = Default::default();
            builder = builder.parent_raw(progman);
            progman.0 = _progman.0;
            builder = builder.parent_raw(progman);
        } */

        let window = builder.build()?;
        Ok(window)
    }

    pub fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.hwnd()?.0))
    }

    pub fn update_position(&self) -> Result<()> {
        let rect = WindowsApi::virtual_screen_rect()?;
        let main_hwnd = self.hwnd()?;

        // Try to position the wallpaper under desktop items
        match Self::try_set_under_desktop_items(main_hwnd) {
            Ok(_) => {
                // When parented to WorkerW, coordinates are relative to parent
                // Calculate dimensions from the absolute screen rect
                let relative_rect = RECT {
                    left: 0,
                    top: 0,
                    right: rect.right - rect.left,
                    bottom: rect.bottom - rect.top,
                };

                WindowsApi::move_window(main_hwnd, &relative_rect)?;
                WindowsApi::set_position(main_hwnd, None, &relative_rect, SWP_ASYNCWINDOWPOS)?;
            }
            Err(e) => {
                log::warn!(
                    "Failed to attach to desktop hierarchy: {}, using absolute positioning",
                    e
                );
                // Fallback to absolute positioning without parent
                WindowsApi::move_window(main_hwnd, &rect)?;
                WindowsApi::set_position(main_hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
                self.window.set_always_on_bottom(true)?;
            }
        }

        log_error!(Self::refresh_desktop().map_err(|e| format!("Failed to refresh desktop: {e}")));
        Ok(())
    }

    /// Sets up the desktop layer by sending a message to Progman to spawn WorkerW
    fn setup_desktop_layer() -> Result<()> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };
        // Send 0x052C to Progman. This message directs Progman to spawn a WorkerW
        // behind the desktop icons. If it is already there, nothing happens.
        unsafe { PostMessageW(Some(progman), 0x052C, WPARAM(0xD), LPARAM(0x1))? };
        Ok(())
    }

    /// Detects the desktop configuration and returns desktop information
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
                // Find next worker after the current one
                if let Ok(_worker) =
                    FindWindowExA(None, Some(current.hwnd()), pcstr!("WorkerW"), None)
                {
                    worker_w = Some(_worker);
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
            // Check if DefView is a child of Progman
            defview = unsafe {
                FindWindowExA(Some(progman), None, pcstr!("SHELLDLL_DefView"), None).ok()
            };

            if defview.is_some() {
                // Check if Progman has WS_EX_NOREDIRECTIONBITMAP style
                let ex_style = WindowsApi::get_ex_styles(progman);
                is_raised_desktop = ex_style.contains(WS_EX_NOREDIRECTIONBITMAP);

                // Find WorkerW as child of Progman
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

    /// Attaches the wallpaper window to the desktop using the appropriate method
    fn attach_to_desktop(hwnd: HWND, desktop_info: &DesktopInfo) -> Result<()> {
        if desktop_info.is_raised_desktop {
            Self::attach_raised_desktop(hwnd, desktop_info)
        } else {
            Self::attach_standard_desktop(hwnd, desktop_info)
        }
    }

    /// Attaches wallpaper to standard desktop (WorkerW parent)
    fn attach_standard_desktop(hwnd: HWND, desktop_info: &DesktopInfo) -> Result<()> {
        let worker_w = desktop_info
            .worker_w
            .ok_or("WorkerW not found for standard desktop")?;

        unsafe {
            // Check if already parented to WorkerW to avoid flicker
            let current_parent = GetParent(hwnd).ok();
            if current_parent != Some(worker_w) {
                SetParent(hwnd, Some(worker_w))?;
            }

            // Position at bottom of z-order
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

    /// Attaches wallpaper to raised desktop with layered shell view (Progman parent)
    fn attach_raised_desktop(hwnd: HWND, desktop_info: &DesktopInfo) -> Result<()> {
        let defview = desktop_info
            .defview
            .ok_or("DefView not found for raised desktop")?;

        unsafe {
            // Check if already parented to Progman to avoid flicker
            let current_parent = GetParent(hwnd).ok();
            let needs_reparent = current_parent != Some(desktop_info.progman);

            if needs_reparent {
                let mut style = WindowsApi::get_styles(hwnd);
                style |= WS_CHILDWINDOW;
                SetWindowLongPtrW(hwnd, GWL_STYLE, style.0 as isize);

                let mut ex_style = WindowsApi::get_ex_styles(hwnd);
                ex_style &= !WS_EX_ACCEPTFILES;
                ex_style &= !WS_EX_APPWINDOW;
                ex_style &= !WS_EX_WINDOWEDGE; // this can be removed wait for https://github.com/tauri-apps/tao/issues/1153
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style.0 as isize);

                // Set parent to Progman
                SetParent(hwnd, Some(desktop_info.progman))?;
            }

            // Position wallpaper below DefView in z-order
            WindowsApi::set_position(
                hwnd,
                Some(defview),
                &Default::default(),
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )?;

            // Ensure WorkerW z-order is correct
            if let Some(worker_w) = desktop_info.worker_w {
                WindowsApi::set_position(
                    worker_w,
                    Some(hwnd),
                    &Default::default(),
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                )?;
            }
        }

        Ok(())
    }

    /// Main function to set wallpaper under desktop items
    fn try_set_under_desktop_items(hwnd: HWND) -> Result<()> {
        // Setup desktop layer
        Self::setup_desktop_layer()?;

        // Detect desktop configuration
        let desktop_info = Self::detect_desktop_info()?;

        // Attach wallpaper to desktop
        Self::attach_to_desktop(hwnd, &desktop_info)?;

        Ok(())
    }

    /// this is only needed on the case 2 of try_set_inside_workerw
    fn refresh_desktop() -> Result<()> {
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
