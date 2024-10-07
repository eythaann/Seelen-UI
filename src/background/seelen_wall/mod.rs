mod hook;

use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::Gdi::{InvalidateRect, UpdateWindow},
    UI::WindowsAndMessaging::{
        FindWindowA, FindWindowExA, PostMessageW, SetParent, SWP_NOACTIVATE,
    },
};

use crate::{
    error_handler::Result,
    log_error, pcstr,
    seelen::get_app_handle,
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
    const TARGET: &str = "seelen-wall";

    pub fn new() -> Result<Self> {
        log::info!("Creating {}", Self::TARGET);
        Ok(Self {
            window: Self::create_window()?,
        })
    }

    fn create_window() -> Result<WebviewWindow> {
        let handle = get_app_handle();
        let window = tauri::WebviewWindowBuilder::new(
            handle,
            Self::TARGET,
            tauri::WebviewUrl::App("seelen_wall/index.html".into()),
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
        // idk why I add this but lively wallpaper has it XD
        // .additional_browser_args("--disk-cache-size=1 --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection")
        .build()?;

        window.set_always_on_bottom(true)?;
        Ok(window)
    }

    pub fn update_position(&self) -> Result<()> {
        let mut rect = WindowsApi::virtual_screen_rect()?;
        let main_hwnd = HWND(self.window.hwnd()?.0);

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
        WindowsApi::set_position(main_hwnd, None, &rect, SWP_NOACTIVATE)?;
        log_error!(Self::refresh_desktop().map_err(|e| format!("Failed to refresh desktop: {}", e)));
        Ok(())
    }

    fn try_set_inside_workerw(hwnd: HWND) -> Result<()> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };

        // Send 0x052C to Progman. This message directs Progman to spawn a WorkerW
        // behind the desktop icons. If it is already there, nothing happens.
        unsafe { PostMessageW(progman, 0x052C, WPARAM(0xD), LPARAM(0x1))? };

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
            if FindWindowExA(current, None, pcstr!("SHELLDLL_DefView"), None).is_ok() {
                // find next worker after the current one
                if let Ok(_worker) = FindWindowExA(None, current, pcstr!("WorkerW"), None) {
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
            worker =
                unsafe { FindWindowExA(progman, HWND::default(), pcstr!("WorkerW"), None).ok() };
            while worker.is_none() && attempts < 10 {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
                worker = unsafe {
                    FindWindowExA(progman, HWND::default(), pcstr!("WorkerW"), None).ok()
                };
            }
        }

        match worker {
            Some(worker) => {
                unsafe { SetParent(hwnd, worker)? };
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
                FindWindowExA(progman, HWND::default(), pcstr!("SHELLDLL_DefView"), None)
            {
                InvalidateRect(shell_view, None, true).ok()?;
                UpdateWindow(shell_view).ok()?;
            }
        }
        Ok(())
    }
}
