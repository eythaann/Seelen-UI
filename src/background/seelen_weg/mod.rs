pub mod handler;
pub mod hook;
pub mod icon_extractor;

use getset::{Getters, MutGetters};
use icon_extractor::extract_and_save_icon;
use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewWindow, Wry};
use win_screenshot::capture::capture_window;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{
        EnumWindows, GetParent, HWND_TOPMOST, SHOW_WINDOW_CMD, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE, SW_SHOWNORMAL, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW
    },
};

use crate::{
    error_handler::Result,
    seelen::{get_app_handle, SEELEN},
    utils::are_overlaped,
    windows_api::{AppBarData, AppBarDataState, WindowsApi},
};

lazy_static! {
    static ref TITLE_BLACK_LIST: Vec<&'static str> = Vec::from([
        "",
        "Task Switching",
        "DesktopWindowXamlSource",
        "SeelenWeg",
        "SeelenWeg Hitbox",
        "SeelenWeg Hitbox",
        "Seelen Window Manager",
        "Seelen Fancy Toolbar",
        "Seelen Fancy Toolbar Hitbox",
        "Program Manager",
    ]);
    static ref EXE_BLACK_LIST: Vec<&'static str> = Vec::from([
        "msedgewebview2.exe",
        "SearchHost.exe",
        "StartMenuExperienceHost.exe",
    ]);
}

#[derive(Debug, Serialize, Clone)]
pub struct SeelenWegApp {
    hwnd: isize,
    exe: String,
    title: String,
    icon_path: String,
    execution_path: String,
    process_hwnd: isize,
}

#[derive(Getters, MutGetters)]
pub struct SeelenWeg {
    handle: AppHandle<Wry>,
    apps: Vec<SeelenWegApp>,
    window: WebviewWindow<Wry>,
    hitbox: WebviewWindow<Wry>,
    #[getset(get = "pub")]
    ready: bool,
    hidden: bool,
    overlaped: bool,
    last_hitbox_rect: Option<RECT>,
}

impl SeelenWeg {
    pub fn new(monitor: isize) -> Result<Self> {
        log::info!("Creating SeelenWeg / {}", monitor);
        let handle = get_app_handle();
        let (window, hitbox) = Self::create_window(&handle, monitor)?;

        let weg = Self {
            handle,
            apps: Vec::new(),
            window,
            hitbox,
            ready: false,
            hidden: false,
            overlaped: false,
            last_hitbox_rect: None,
        };

        Ok(weg)
    }

    pub fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    pub fn set_active_window(&self, hwnd: HWND) -> Result<()> {
        if WindowsApi::get_window_text(hwnd) == "Task Switching" {
            return Ok(());
        }
        self.emit("set-focused-handle", hwnd.0)?;
        Ok(())
    }

    pub fn missing_icon(&self) -> String {
        self.handle
            .path()
            .resolve("static/icons/missing.png", BaseDirectory::Resource)
            .expect("Failed to resolve default icon path")
            .to_string_lossy()
            .to_uppercase()
    }

    pub fn extract_icon(&self, exe_path: &str) -> Result<String> {
        Ok(extract_and_save_icon(&self.handle, exe_path)?
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string())
    }

    pub fn contains_app(&self, hwnd: HWND) -> bool {
        self.apps
            .iter()
            .any(|app| app.hwnd == hwnd.0 || app.process_hwnd == hwnd.0)
    }

    pub fn update_app(&mut self, hwnd: HWND) {
        let app = self.apps.iter_mut().find(|app| app.hwnd == hwnd.0);
        if let Some(app) = app {
            app.title = WindowsApi::get_window_text(hwnd);
            self.window
                .emit("update-open-app-info", app.clone())
                .expect("Failed to emit");
        }
    }

    pub fn replace_hwnd(&mut self, old: HWND, new: HWND) -> Result<()> {
        let mut found = None;
        for app in self.apps.iter_mut() {
            if app.hwnd == old.0 {
                app.hwnd = new.0;
                found = Some(app.clone());
                break;
            }
        }

        if let Some(app) = found {
            self.emit("replace-open-app", app)?;
        }

        Ok(())
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) {
        if self.contains_app(hwnd) {
            return;
        }

        log::trace!(
            "Adding {} <=> {:?}",
            hwnd.0,
            WindowsApi::get_window_text(hwnd)
        );

        let exe_path = WindowsApi::exe_path(hwnd).unwrap_or_default();
        let mut icon_path = self.missing_icon();
        if exe_path != "" {
            icon_path = self.extract_icon(&exe_path).unwrap_or(icon_path);
        }

        let app = SeelenWegApp {
            hwnd: hwnd.0,
            exe: exe_path.clone(),
            title: WindowsApi::get_window_text(hwnd),
            icon_path,
            execution_path: exe_path,
            process_hwnd: hwnd.0,
        };

        self.window
            .emit("add-open-app", app.clone())
            .expect("Failed to emit");

        self.apps.push(app);
    }

    pub fn remove_hwnd(&mut self, hwnd: HWND) {
        self.apps.retain(|app| app.hwnd != hwnd.0);
        self.window
            .emit("remove-open-app", hwnd.0)
            .expect("Failed to emit");
    }

    pub fn is_overlapping(&self, hwnd: HWND) -> bool {
        let rect = WindowsApi::get_window_rect(hwnd);
        let hitbox_rect = self.last_hitbox_rect.unwrap_or_else(|| {
            WindowsApi::get_window_rect(HWND(
                self.hitbox.hwnd().expect("Failed to get hitbox handle").0,
            ))
        });
        are_overlaped(&hitbox_rect, &rect)
    }

    pub fn update_status_if_needed(&mut self, hwnd: HWND) -> Result<()> {
        if !self.ready
            || !WindowsApi::is_window_visible(hwnd)
            || WindowsApi::is_iconic(hwnd)
            || TITLE_BLACK_LIST.contains(&WindowsApi::get_window_text(hwnd).as_str())
            || EXE_BLACK_LIST.contains(&WindowsApi::exe(hwnd).unwrap_or_default().as_str())
        {
            return Ok(());
        }

        let last_status = self.overlaped.clone();
        self.overlaped = self.is_overlapping(hwnd);
        if last_status == self.overlaped {
            return Ok(());
        }

        self.last_hitbox_rect = if self.overlaped {
            Some(WindowsApi::get_window_rect(HWND(self.hitbox.hwnd()?.0)))
        } else {
            None
        };

        self.emit("set-auto-hide", self.overlaped)?;
        Ok(())
    }

    pub fn ensure_hitbox_zorder(&self) -> Result<()> {
        WindowsApi::bring_to(HWND(self.window.hwnd()?.0), HWND_TOPMOST)
    }

    pub fn is_real_window(hwnd: HWND, ignore_frame: bool) -> bool {
        if !WindowsApi::is_window_visible(hwnd) {
            return false;
        }

        let parent = unsafe { GetParent(hwnd) };
        if parent.0 != 0 {
            return false;
        }

        let ex_style = WindowsApi::get_ex_styles(hwnd);
        if (ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE))
            && !ex_style.contains(WS_EX_APPWINDOW)
        {
            return false;
        }

        let exe_path = WindowsApi::exe_path(hwnd).unwrap_or_default();
        if exe_path.starts_with("C:\\Windows\\SystemApps")
            || (!ignore_frame && exe_path.ends_with("ApplicationFrameHost.exe"))
        {
            return false;
        }

        let title = WindowsApi::get_window_text(hwnd);
        !TITLE_BLACK_LIST.contains(&title.as_str())
    }

    pub fn capture_window(hwnd: HWND) -> Option<DynamicImage> {
        capture_window(hwnd.0).ok().map(|buf| {
            let image = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap_or_default();
            DynamicImage::ImageRgba8(image)
        })
    }

    pub fn hide(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.window.hwnd()?, SW_HIDE)?;
        WindowsApi::show_window_async(self.hitbox.hwnd()?, SW_HIDE)?;
        self.hidden = true;
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.window.hwnd()?, SW_SHOWNOACTIVATE)?;
        WindowsApi::show_window_async(self.hitbox.hwnd()?, SW_SHOWNOACTIVATE)?;
        self.hidden = false;
        Ok(())
    }
}

impl SeelenWeg {
    const TARGET: &'static str = "seelenweg";
    const TARGET_HITBOX: &'static str = "seelenweg-hitbox";

    fn create_window(
        manager: &AppHandle<Wry>,
        monitor_id: isize,
    ) -> Result<(WebviewWindow, WebviewWindow)> {
        let monitor_info = WindowsApi::monitor_info(HMONITOR(monitor_id))?;
        let rc_work = monitor_info.monitorInfo.rcWork;

        let hitbox = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            manager,
            format!("{}/{}", Self::TARGET_HITBOX, monitor_id),
            tauri::WebviewUrl::App("seelenweg-hitbox/index.html".into()),
        )
        .title("SeelenWeg Hitbox")
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()?;

        let window = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            manager,
            format!("{}/{}", Self::TARGET, monitor_id),
            tauri::WebviewUrl::App("seelenweg/index.html".into()),
        )
        .title("SeelenWeg")
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()?;

        window.set_ignore_cursor_events(true)?;

        let main_hwnd = HWND(window.hwnd()?.0);
        let hitbox_hwnd = HWND(hitbox.hwnd()?.0);

        WindowsApi::move_window(hitbox_hwnd, &rc_work)?;

        // pre set position before resize in case of multiples dpi
        WindowsApi::move_window(main_hwnd, &rc_work)?;
        WindowsApi::set_position(main_hwnd, None, &rc_work, SWP_NOACTIVATE)?;

        window.once("complete-setup", move |_event| {
            std::thread::spawn(move || {
                if let Some(monitor) = SEELEN.lock().monitor_by_id_mut(monitor_id) {
                    if let Some(weg) = monitor.weg_mut() {
                        weg.ready = true;
                    }
                }
            });
        });

        Ok((window, hitbox))
    }

    pub fn hide_taskbar(hide: bool) -> Result<()> {
        let state: AppBarDataState;
        let cmdshow: SHOW_WINDOW_CMD;

        if hide {
            state = AppBarDataState::AutoHide;
            cmdshow = SW_HIDE;
        } else {
            state = AppBarDataState::AlwaysOnTop;
            cmdshow = SW_SHOWNORMAL;
        }

        let handles = get_taskbars_handles()?;
        for handle in handles {
            let pdata = AppBarData::from_handle(handle);
            pdata.set_state(state);
            WindowsApi::show_window(handle, cmdshow)?;
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref FOUNDS: Mutex<Vec<HWND>> = Mutex::new(Vec::new());
    pub static ref TASKBAR_CLASS: Vec<&'static str> =
        Vec::from(["Shell_TrayWnd", "Shell_SecondaryTrayWnd",]);
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
    let class = WindowsApi::get_class(hwnd).unwrap_or_default();
    if TASKBAR_CLASS.contains(&class.as_str()) {
        FOUNDS.lock().push(hwnd);
    }
    true.into()
}

pub fn get_taskbars_handles() -> Result<Vec<HWND>> {
    unsafe { EnumWindows(Some(enum_windows_proc), LPARAM(0))? };
    let mut found = FOUNDS.lock();
    let result = found.clone();
    found.clear();
    Ok(result)
}
