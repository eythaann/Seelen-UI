pub mod handler;
pub mod hook;
pub mod icon_extractor;

use std::thread::JoinHandle;

use getset::{Getters, MutGetters};
use icon_extractor::extract_and_save_icon;
use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::state::AppExtraFlag;
use serde::Serialize;
use tauri::{path::BaseDirectory, Emitter, Listener, Manager, WebviewWindow, Wry};
use win_screenshot::capture::capture_window;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    UI::WindowsAndMessaging::{
        EnumWindows, GetParent, HWND_TOPMOST, SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE,
        WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    },
};

use crate::{
    error_handler::Result,
    log_error,
    modules::uwp::UWP_MANAGER,
    seelen::{get_app_handle, SEELEN},
    seelen_bar::FancyToolbar,
    state::application::FULL_STATE,
    trace_lock,
    utils::{are_overlaped, sleep_millis},
    windows_api::{AppBarData, AppBarDataState, WindowsApi},
};

lazy_static! {
    static ref TITLE_BLACK_LIST: Vec<&'static str> = Vec::from([
        "",
        "Task Switching",
        "DesktopWindowXamlSource",
        "SeelenWeg",
        "SeelenWeg Hitbox",
        "Seelen Window Manager",
        "Seelen Fancy Toolbar",
        "Seelen Fancy Toolbar Hitbox",
        "Program Manager",
    ]);
    static ref OPEN_APPS: Mutex<Vec<SeelenWegApp>> = Mutex::new(Vec::new());
}

static OVERLAP_BLACK_LIST_BY_TITLE: [&str; 7] = [
    "",
    "SeelenWeg",
    "SeelenWeg Hitbox",
    "Seelen Window Manager",
    "Seelen Fancy Toolbar",
    "Seelen Fancy Toolbar Hitbox",
    "Program Manager",
];

static OVERLAP_BLACK_LIST_BY_EXE: [&str; 4] = [
    "msedgewebview2.exe",
    "SearchHost.exe",
    "StartMenuExperienceHost.exe",
    "ShellExperienceHost.exe",
];

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
    window: WebviewWindow<Wry>,
    hitbox: WebviewWindow<Wry>,
    #[getset(get = "pub")]
    ready: bool,
    hidden: bool,
    overlaped: bool,
    last_hitbox_rect: Option<RECT>,
}

impl Drop for SeelenWeg {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        log_error!(self.window.destroy());
        log_error!(self.hitbox.destroy());
    }
}

// SINGLETON
impl SeelenWeg {
    pub fn set_active_window(hwnd: HWND) -> Result<()> {
        if WindowsApi::get_window_text(hwnd) == "Task Switching" {
            return Ok(());
        }

        let handle = get_app_handle();
        handle.emit("set-focused-handle", hwnd.0)?;
        handle.emit(
            "set-focused-executable",
            WindowsApi::exe(hwnd).unwrap_or_default(),
        )?;
        Ok(())
    }

    pub fn missing_icon() -> String {
        get_app_handle()
            .path()
            .resolve("static/icons/missing.png", BaseDirectory::Resource)
            .expect("Failed to resolve default icon path")
            .to_string_lossy()
            .to_uppercase()
    }

    pub fn extract_icon(exe_path: &str) -> Result<String> {
        Ok(extract_and_save_icon(&get_app_handle(), exe_path)?
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string())
    }

    pub fn contains_app(hwnd: HWND) -> bool {
        OPEN_APPS
            .lock()
            .iter()
            .any(|app| app.hwnd == hwnd.0 || app.process_hwnd == hwnd.0)
    }

    pub fn update_app(hwnd: HWND) {
        let mut apps = OPEN_APPS.lock();
        let app = apps.iter_mut().find(|app| app.hwnd == hwnd.0);
        if let Some(app) = app {
            app.title = WindowsApi::get_window_text(hwnd);
            get_app_handle()
                .emit("update-open-app-info", app.clone())
                .expect("Failed to emit");
        }
    }

    pub fn replace_hwnd(old: HWND, new: HWND) -> Result<()> {
        let mut found = None;
        let mut apps = OPEN_APPS.lock();
        for app in apps.iter_mut() {
            if app.hwnd == old.0 {
                app.hwnd = new.0;
                found = Some(app.clone());
                break;
            }
        }

        if let Some(app) = found {
            get_app_handle().emit("replace-open-app", app)?;
        }

        Ok(())
    }

    pub fn add_hwnd(hwnd: HWND) {
        if Self::contains_app(hwnd) {
            return;
        }

        let title = WindowsApi::get_window_text(hwnd);

        if let Some(config) = FULL_STATE.load().get_app_config_by_window(hwnd) {
            if config.options.contains(&AppExtraFlag::Hidden) {
                log::trace!("Skipping by config: {} <=> {}", hwnd.0, title);
                return;
            }
        }

        log::trace!("Adding {} <=> {}", hwnd.0, title);
        let mut app = SeelenWegApp {
            hwnd: hwnd.0,
            exe: String::new(),
            title,
            icon_path: String::new(),
            execution_path: String::new(),
            process_hwnd: hwnd.0,
        };

        if let Ok(path) = WindowsApi::exe_path_v2(hwnd) {
            app.exe = path.to_string_lossy().to_string();
            app.icon_path = if !app.exe.is_empty() {
                Self::extract_icon(&app.exe).unwrap_or_else(|_| Self::missing_icon())
            } else {
                Self::missing_icon()
            };

            let exe = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            app.execution_path = match trace_lock!(UWP_MANAGER).get_from_path(&path) {
                Some(package) => package
                    .get_shell_path(&exe)
                    .unwrap_or_else(|| app.exe.clone()),
                None => app.exe.clone(),
            };
        }

        get_app_handle()
            .emit("add-open-app", app.clone())
            .expect("Failed to emit");

        OPEN_APPS.lock().push(app);
    }

    pub fn remove_hwnd(hwnd: HWND) {
        OPEN_APPS.lock().retain(|app| app.hwnd != hwnd.0);
        get_app_handle()
            .emit("remove-open-app", hwnd.0)
            .expect("Failed to emit");
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
}

// INSTANCE
impl SeelenWeg {
    pub fn new(postfix: &str) -> Result<Self> {
        log::info!("Creating {}/{}", Self::TARGET, postfix);
        let (window, hitbox) = Self::create_window(postfix)?;

        let weg = Self {
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

    pub fn is_overlapping(&self, hwnd: HWND) -> bool {
        let rect = WindowsApi::get_window_rect_without_margins(hwnd);
        let hitbox_rect = self.last_hitbox_rect.unwrap_or_else(|| {
            WindowsApi::get_window_rect_without_margins(HWND(
                self.hitbox.hwnd().expect("Failed to get hitbox handle").0,
            ))
        });
        are_overlaped(&hitbox_rect, &rect)
    }

    pub fn set_overlaped_status(&mut self, is_overlaped: bool) -> Result<()> {
        if self.overlaped == is_overlaped {
            return Ok(());
        }

        self.overlaped = is_overlaped;
        self.last_hitbox_rect = if self.overlaped {
            Some(WindowsApi::get_window_rect_without_margins(HWND(
                self.hitbox.hwnd()?.0,
            )))
        } else {
            None
        };

        self.emit("set-auto-hide", self.overlaped)?;
        Ok(())
    }

    pub fn handle_overlaped_status(&mut self, hwnd: HWND) -> Result<()> {
        let should_handle_hidden = self.ready
            && WindowsApi::is_window_visible(hwnd)
            && !OVERLAP_BLACK_LIST_BY_TITLE.contains(&WindowsApi::get_window_text(hwnd).as_str())
            && !OVERLAP_BLACK_LIST_BY_EXE
                .contains(&WindowsApi::exe(hwnd).unwrap_or_default().as_str());

        if !should_handle_hidden {
            return Ok(());
        }

        self.set_overlaped_status(self.is_overlapping(hwnd))
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

    pub fn ensure_hitbox_zorder(&self) -> Result<()> {
        WindowsApi::bring_to(self.hitbox.hwnd()?, HWND_TOPMOST)?;
        self.set_positions(WindowsApi::monitor_from_window(self.window.hwnd()?).0)?;
        Ok(())
    }

    pub fn set_positions(&self, monitor_id: isize) -> Result<()> {
        let rc_work = FancyToolbar::get_work_area_by_monitor(monitor_id)?;
        let main_hwnd = HWND(self.window.hwnd()?.0);
        // pre set position before resize in case of multiples dpi
        WindowsApi::move_window(main_hwnd, &rc_work)?;
        WindowsApi::set_position(main_hwnd, None, &rc_work, SWP_NOACTIVATE)?;
        Ok(())
    }
}

impl SeelenWeg {
    const TARGET: &'static str = "seelenweg";
    const TARGET_HITBOX: &'static str = "seelenweg-hitbox";

    fn create_window(postfix: &str) -> Result<(WebviewWindow, WebviewWindow)> {
        let manager = get_app_handle();

        let hitbox = tauri::WebviewWindowBuilder::new(
            &manager,
            format!("{}/{}", Self::TARGET_HITBOX, postfix),
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
        .drag_and_drop(false)
        .build()?;

        let window = tauri::WebviewWindowBuilder::new(
            &manager,
            format!("{}/{}", Self::TARGET, postfix),
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
        .drag_and_drop(false)
        .owner(&hitbox)?
        .build()?;

        window.set_ignore_cursor_events(true)?;

        let postfix = postfix.to_string();
        window.once("complete-setup", move |_event| {
            std::thread::spawn(move || {
                if let Some(monitor) = trace_lock!(SEELEN).monitor_by_name_mut(&postfix) {
                    if let Some(weg) = monitor.weg_mut() {
                        weg.ready = true;
                    }
                }
            });
        });

        let label = window.label().to_string();
        window.listen("request-all-open-apps", move |_| {
            let handler = get_app_handle();
            let apps = &*OPEN_APPS.lock();
            log_error!(handler.emit_to(&label, "add-multiple-open-apps", apps));
        });
        Ok((window, hitbox))
    }

    pub fn hide_taskbar() -> JoinHandle<()> {
        std::thread::spawn(move || match get_taskbars_handles() {
            Ok(handles) => {
                let mut attempts = 0;
                while attempts < 10 && FULL_STATE.load().is_weg_enabled() {
                    for handle in &handles {
                        AppBarData::from_handle(*handle).set_state(AppBarDataState::AutoHide);
                        let _ = WindowsApi::show_window(*handle, SW_HIDE);
                    }
                    attempts += 1;
                    sleep_millis(50);
                }
            }
            Err(err) => log::error!("Failed to get taskbars handles: {:?}", err),
        })
    }

    pub fn show_taskbar() -> Result<()> {
        for hwnd in get_taskbars_handles()? {
            AppBarData::from_handle(hwnd).set_state(AppBarDataState::AlwaysOnTop);
            WindowsApi::show_window(hwnd, SW_SHOWNOACTIVATE)?;
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
