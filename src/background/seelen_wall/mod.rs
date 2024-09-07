use tauri::WebviewWindow;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    Graphics::Gdi::HMONITOR,
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

// statics
impl SeelenWall {
    pub const TITLE: &str = "Seelen Wall";
    const TARGET: &str = "seelen-wall";

    pub fn set_position(&self, monitor: HMONITOR) -> Result<()> {
        let monitor_info = WindowsApi::monitor_info(monitor)?;
        let rc_monitor = monitor_info.monitorInfo.rcMonitor;
        let main_hwnd = HWND(self.window.hwnd()?.0);

        // pre set position for resize in case of multiples dpi
        WindowsApi::move_window(main_hwnd, &rc_monitor)?;
        WindowsApi::set_position(main_hwnd, None, &rc_monitor, SWP_NOACTIVATE)?;
        std::thread::spawn(move || log_error!(Self::try_set_under_progman(main_hwnd)));
        Ok(())
    }

    fn try_set_under_progman(hwnd: HWND) -> Result<()> {
        let progman = unsafe { FindWindowA(pcstr!("Progman"), None) };
        if progman.0 == 0 {
            return Err("Failed to find progman window".into());
        }

        // Send 0x052C to Progman. This message directs Progman to spawn a WorkerW
        // behind the desktop icons. If it is already there, nothing happens.
        unsafe { PostMessageW(progman, 0x052C, WPARAM(0xD), LPARAM(0x1))? };

        let mut worker = unsafe { FindWindowExA(progman, HWND(0), pcstr!("WorkerW"), None) };
        let mut attempts = 0;
        while worker.0 == 0 && attempts < 10 {
            attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            worker = unsafe { FindWindowExA(progman, HWND(0), pcstr!("WorkerW"), None) };
        }

        if worker.0 == 0 {
            return Err("Failed to find/create progman worker window".into());
        }
        unsafe { SetParent(hwnd, worker) };
        Ok(())
    }

    fn create_window(postfix: &str) -> Result<WebviewWindow> {
        let handle = get_app_handle();
        let window = tauri::WebviewWindowBuilder::new(
            &handle,
            format!("{}/{}", Self::TARGET, postfix),
            tauri::WebviewUrl::App("seelen_wall/index.html".into()),
        )
        .title(Self::TITLE)
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .closable(false)
        .decorations(false)
        .shadow(false)
        .disable_drag_drop_handler()
        .skip_taskbar(true)
        // idk why I add this but lively wallpaper has it XD
        // .additional_browser_args("--disk-cache-size=1 --disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection")
        .build()?;

        window.set_always_on_bottom(true)?;
        Ok(window)
    }
}

impl SeelenWall {
    pub fn new(postfix: &str) -> Result<Self> {
        Ok(Self {
            window: Self::create_window(postfix)?,
        })
    }
}
