mod hook;

use base64::Engine;
use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::Gdi::{InvalidateRect, UpdateWindow},
    UI::WindowsAndMessaging::{
        FindWindowA, FindWindowExA, PostMessageW, SetParent, SWP_ASYNCWINDOWPOS,
    },
};

use crate::{
    app::get_app_handle,
    error::Result,
    log_error, pcstr,
    widgets::WebviewArgs,
    windows_api::{WindowEnumerator, WindowsApi},
};

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

        let window = tauri::WebviewWindowBuilder::new(
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
        .visible(false)
        .disable_drag_drop_handler()
        .skip_taskbar(true)
        .data_directory(args.data_directory())
        .additional_browser_args(&args.to_string())
        .build()?;

        window.set_always_on_bottom(true)?;
        Ok(window)
    }

    pub fn hwnd(&self) -> Result<HWND> {
        Ok(HWND(self.window.hwnd()?.0))
    }

    pub fn update_position(&self) -> Result<()> {
        let mut rect = WindowsApi::virtual_screen_rect()?;
        let main_hwnd = self.hwnd()?;

        if Self::try_set_inside_workerw(main_hwnd).is_ok() {
            // rect relative to the parent
            rect = RECT {
                top: 0,
                left: 0,
                right: rect.right - rect.left,
                bottom: rect.bottom - rect.top,
            };
        }

        // pre set position for resize in case of multiples dpi
        WindowsApi::move_window(main_hwnd, &rect)?;
        WindowsApi::set_position(main_hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
        log_error!(Self::refresh_desktop().map_err(|e| format!("Failed to refresh desktop: {e}")));
        Ok(())
    }

    fn try_set_inside_workerw(hwnd: HWND) -> Result<()> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };

        // Send 0x052C to Progman. This message directs Progman to spawn a WorkerW
        // behind the desktop icons. If it is already there, nothing happens.
        unsafe { PostMessageW(Some(progman), 0x052C, WPARAM(0xD), LPARAM(0x1))? };

        // CASE 1:
        // 0x00010190 "" WorkerW
        //   ...
        //   0x000100EE "" SHELLDLL_DefView
        //     0x000100F0 "FolderView" SysListView32
        // 0x00100B8A "" WorkerW       <-- This is the WorkerW instance we are after!
        // 0x000100EC "Program Manager" Progman
        let mut worker = None;

        WindowEnumerator::new().for_each(|current| unsafe {
            // check if current contains SHELLDLL_DefView
            if FindWindowExA(Some(current.hwnd()), None, pcstr!("SHELLDLL_DefView"), None).is_ok() {
                // find next worker after the current one
                if let Ok(_worker) =
                    FindWindowExA(None, Some(current.hwnd()), pcstr!("WorkerW"), None)
                {
                    worker = Some(_worker);
                }
            }
        })?;

        // CASE 2:
        // Some Windows 11 builds have a different Progman window layout.
        // If the above code failed to find WorkerW, we should try this.
        // 0x000100EC "Program Manager" Progman
        //   0x000100EE "" SHELLDLL_DefView
        //     0x000100F0 "FolderView" SysListView32
        //   0x00100B8A "" WorkerW       <-- This is the WorkerW instance we are after!
        if worker.is_none() {
            let mut attempts = 0;
            worker = unsafe { FindWindowExA(Some(progman), None, pcstr!("WorkerW"), None).ok() };
            while worker.is_none() && attempts < 10 {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
                worker =
                    unsafe { FindWindowExA(Some(progman), None, pcstr!("WorkerW"), None).ok() };
            }
        }

        match worker {
            Some(worker) => {
                unsafe { SetParent(hwnd, Some(worker))? };
                Ok(())
            }
            None => Err("Failed to find/create progman worker window".into()),
        }
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
