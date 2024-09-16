use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    UI::WindowsAndMessaging::{
        FindWindowA, FindWindowExA, PostMessageW, SetParent, SWP_NOACTIVATE,
    },
};

use crate::{
    error_handler::Result, log_error, pcstr, seelen::get_app_handle, windows_api::WindowsApi,
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
            &handle,
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

    pub fn set_position(&self) -> Result<()> {
        let mut rect = WindowsApi::virtual_screen_rect()?;
        let main_hwnd = HWND(self.window.hwnd()?.0);

        if Self::try_set_under_progman(main_hwnd).is_ok() {
            // rect relative to the parent
            rect = RECT {
                top: 0,
                left: 0,
                right: rect.right - rect.left,
                bottom: rect.bottom - rect.top,
            }
        }

        // pre set position for resize in case of multiples dpi
        WindowsApi::move_window(main_hwnd, &rect)?;
        WindowsApi::set_position(main_hwnd, None, &rect, SWP_NOACTIVATE)?;
        Ok(())
    }

    fn try_set_under_progman(hwnd: HWND) -> Result<()> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None)? };

        // Send 0x052C to Progman. This message directs Progman to spawn a WorkerW
        // behind the desktop icons. If it is already there, nothing happens.
        unsafe { PostMessageW(progman, 0x052C, WPARAM(0xD), LPARAM(0x1))? };

        let mut worker =
            unsafe { FindWindowExA(progman, HWND::default(), pcstr!("WorkerW"), None) };
        let mut attempts = 0;
        while worker.is_err() && attempts < 10 {
            attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            worker = unsafe { FindWindowExA(progman, HWND::default(), pcstr!("WorkerW"), None) };
        }

        match worker {
            Ok(w) => {
                unsafe { SetParent(hwnd, w)? };
                Ok(())
            }
            Err(_) => Err("Failed to find/create progman worker window".into()),
        }
    }
}
