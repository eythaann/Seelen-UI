pub mod cli;
pub mod handler;
pub mod hook;
pub mod icon_extractor;

use std::{collections::HashMap, path::PathBuf, thread::JoinHandle};

use base64::Engine;
use getset::{Getters, MutGetters};
use icon_extractor::{extract_and_save_icon_from_file, extract_and_save_icon_umid};
use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::{
    handlers::SeelenEvent,
    state::{AppExtraFlag, HideMode, SeelenWegSide},
};
use serde::Serialize;
use tauri::{Emitter, Listener, WebviewWindow, Wry};
use win_screenshot::capture::capture_window;
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::HMONITOR,
    UI::WindowsAndMessaging::{
        SWP_NOACTIVATE, SW_HIDE, SW_SHOWNOACTIVATE, SW_SHOWNORMAL, WS_EX_APPWINDOW,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    },
};

use crate::{
    error_handler::Result,
    log_error,
    seelen::get_app_handle,
    seelen_bar::FancyToolbar,
    state::application::FULL_STATE,
    trace_lock,
    utils::{
        are_overlaped,
        constants::{Icons, NATIVE_UI_POPUP_CLASSES, OVERLAP_BLACK_LIST_BY_EXE},
        sleep_millis,
    },
    windows_api::{window::Window, AppBarData, AppBarDataState, WindowEnumerator, WindowsApi},
};

lazy_static! {
    static ref TITLE_BLACK_LIST: Vec<&'static str> = Vec::from([
        "",
        "Task Switching",
        "DesktopWindowXamlSource",
        "Program Manager",
    ]);
    static ref OPEN_APPS: Mutex<Vec<SeelenWegApp>> = Mutex::new(Vec::new());
}

#[derive(Debug, Serialize, Clone)]
pub struct SeelenWegApp {
    hwnd: isize,
    exe: PathBuf,
    title: String,
    icon_path: PathBuf,
    execution_path: String,
    creator_hwnd: isize,
}

#[derive(Getters, MutGetters)]
pub struct SeelenWeg {
    window: WebviewWindow<Wry>,
    overlaped: bool,
    last_overlapped_window: Option<Window>,
    /// Is the rect that the dock should have when it isn't hidden
    pub theoretical_rect: RECT,
}

impl Drop for SeelenWeg {
    fn drop(&mut self) {
        log::info!("Dropping {}", self.window.label());
        if let Ok(hwnd) = self.window.hwnd() {
            AppBarData::from_handle(hwnd).unregister_bar();
        }
        log_error!(self.window.destroy());
    }
}

// SINGLETON
impl SeelenWeg {
    pub fn set_active_window(hwnd: HWND) -> Result<()> {
        let handle = get_app_handle();
        handle.emit(SeelenEvent::WegSetFocusedHandle, hwnd.0 as isize)?;
        handle.emit(
            SeelenEvent::WegSetFocusedExecutable,
            WindowsApi::exe(hwnd).unwrap_or_default(),
        )?;
        Ok(())
    }

    pub fn contains_app(hwnd: HWND) -> bool {
        let addr = hwnd.0 as isize;
        trace_lock!(OPEN_APPS)
            .iter()
            .any(|app| app.hwnd == addr || app.creator_hwnd == addr)
    }

    pub fn update_app(hwnd: HWND) {
        let addr = hwnd.0 as isize;
        let mut apps = trace_lock!(OPEN_APPS);
        let app = apps.iter_mut().find(|app| app.hwnd == addr);
        if let Some(app) = app {
            app.title = WindowsApi::get_window_text(hwnd);
            get_app_handle()
                .emit(SeelenEvent::WegUpdateOpenAppInfo, app.clone())
                .expect("Failed to emit");
        }
    }

    pub fn enumerate_all_windows() -> Result<()> {
        WindowEnumerator::new().for_each(|hwnd| {
            if Self::should_be_added(hwnd) {
                log_error!(Self::add_hwnd(hwnd));
            }
        })
    }

    pub fn add_hwnd(hwnd: HWND) -> Result<()> {
        if Self::contains_app(hwnd) {
            return Ok(());
        }

        let window = Window::from(hwnd);
        let creator = match window.get_frame_creator() {
            Ok(None) => return Ok(()),
            Ok(Some(creator)) => creator,
            Err(_) => window,
        };

        let program_path = creator.exe()?;
        let mut app = SeelenWegApp {
            hwnd: hwnd.0 as isize,
            title: creator.title(),
            exe: program_path.clone(),
            execution_path: program_path.to_string_lossy().to_string(),
            icon_path: Default::default(),
            creator_hwnd: creator.hwnd().0 as isize,
        };

        if let Some(umid) = creator.app_user_model_id() {
            if umid.contains("!") {
                app.execution_path = format!("shell:AppsFolder\\{umid}");
                app.icon_path =
                    extract_and_save_icon_umid(&umid).unwrap_or_else(|_| Icons::missing_app())
            } else {
                app.icon_path = extract_and_save_icon_from_file(program_path)
                    .unwrap_or_else(|_| Icons::missing_app());
            }
        } else {
            app.icon_path = extract_and_save_icon_from_file(program_path)
                .unwrap_or_else(|_| Icons::missing_app());
        }

        get_app_handle()
            .emit(SeelenEvent::WegAddOpenApp, app.clone())
            .expect("Failed to emit");

        trace_lock!(OPEN_APPS).push(app);
        Ok(())
    }

    pub fn remove_hwnd(hwnd: HWND) {
        let addr = hwnd.0 as isize;
        trace_lock!(OPEN_APPS).retain(|app| app.hwnd != addr);
        get_app_handle()
            .emit(SeelenEvent::WegRemoveOpenApp, addr)
            .expect("Failed to emit");
    }

    pub fn should_be_added(hwnd: HWND) -> bool {
        let window = Window::from(hwnd);
        let path = match window.process().program_path() {
            Ok(path) => path,
            Err(_) => return false,
        };

        if !window.is_visible()
            || path.starts_with("C:\\Windows\\SystemApps")
            || path.starts_with("C:\\Windows\\ImmersiveControlPanel")
            || window.parent().is_some()
            || window.is_seelen_overlay()
        {
            return false;
        }

        // this class is used for edge tabs to be shown as independent windows on alt + tab
        // this only applies when the new tab is created it is binded to explorer.exe for some reason
        // maybe we can search/learn more about edge tabs later.
        if window.class() == "Windows.Internal.Shell.TabProxyWindow" {
            return false;
        }

        let ex_style = WindowsApi::get_ex_styles(hwnd);
        if (ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE))
            && !ex_style.contains(WS_EX_APPWINDOW)
        {
            return false;
        }

        if let Ok(frame_creator) = window.get_frame_creator() {
            if frame_creator.is_none() {
                return false;
            }
        }

        if WindowsApi::window_is_uwp_suspended(hwnd).unwrap_or_default() {
            return false;
        }

        if let Some(config) = FULL_STATE.load().get_app_config_by_window(hwnd) {
            if config.options.contains(&AppExtraFlag::Hidden) {
                log::trace!("Skipping by config: {:?}", window);
                return false;
            }
        }

        !TITLE_BLACK_LIST.contains(&window.title().as_str())
    }

    pub fn capture_window(hwnd: HWND) -> Option<DynamicImage> {
        capture_window(hwnd.0 as isize).ok().map(|buf| {
            let image = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap_or_default();
            DynamicImage::ImageRgba8(image)
        })
    }
}

// INSTANCE
impl SeelenWeg {
    pub fn new(postfix: &str) -> Result<Self> {
        let weg = Self {
            window: Self::create_window(postfix)?,
            overlaped: false,
            last_overlapped_window: None,
            theoretical_rect: RECT::default(),
        };
        Ok(weg)
    }

    fn emit<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
        self.window.emit_to(self.window.label(), event, payload)?;
        Ok(())
    }

    fn is_overlapping(&self, hwnd: HWND) -> Result<bool> {
        let window_rect = WindowsApi::get_inner_window_rect(hwnd)?;
        Ok(are_overlaped(&self.theoretical_rect, &window_rect))
    }

    fn set_overlaped_status(&mut self, is_overlaped: bool) -> Result<()> {
        if self.overlaped == is_overlaped {
            return Ok(());
        }
        self.overlaped = is_overlaped;
        self.emit(SeelenEvent::WegOverlaped, self.overlaped)?;
        Ok(())
    }

    pub fn handle_overlaped_status(&mut self, hwnd: HWND) -> Result<()> {
        let window = Window::from(hwnd);
        let monitor = window.monitor();
        let is_overlaped = self.is_overlapping(hwnd)?
            && !window.is_desktop()
            && !window.is_seelen_overlay()
            && !NATIVE_UI_POPUP_CLASSES.contains(&window.class().as_str())
            && !OVERLAP_BLACK_LIST_BY_EXE
                .contains(&WindowsApi::exe(hwnd).unwrap_or_default().as_str());

        let state = FULL_STATE.load();
        let settings = &state.settings().seelenweg;

        if settings.use_multi_monitor_overlap_logic {
            if is_overlaped {
                self.last_overlapped_window = Some(window);
            } else if let Some(past_window) = self.last_overlapped_window {
                if past_window != window
                    && past_window.monitor() != monitor
                    && Window::from(self.window.hwnd()?).monitor() != monitor
                {
                    return Ok(());
                }
            }
        } else {
            self.last_overlapped_window = None;
        }

        self.set_overlaped_status(is_overlaped)
    }

    pub fn hide(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.window.hwnd()?, SW_HIDE)?;
        self.window.emit_to(
            self.window.label(),
            SeelenEvent::HandleLayeredHitboxes,
            false,
        )?;
        Ok(())
    }

    pub fn show(&mut self) -> Result<()> {
        WindowsApi::show_window_async(self.window.hwnd()?, SW_SHOWNOACTIVATE)?;
        self.window.emit_to(
            self.window.label(),
            SeelenEvent::HandleLayeredHitboxes,
            true,
        )?;
        Ok(())
    }

    pub fn set_position(&mut self, monitor: HMONITOR) -> Result<()> {
        let rc_work = FancyToolbar::get_work_area_by_monitor(monitor)?;
        let hwnd = HWND(self.window.hwnd()?.0);

        let state = FULL_STATE.load();
        let settings = &state.settings().seelenweg;
        let monitor_dpi = WindowsApi::get_device_pixel_ratio(monitor)?;
        let total_size = (settings.total_size() as f32 * monitor_dpi) as i32;

        self.theoretical_rect = rc_work;
        let mut hidden_rect = rc_work;
        match settings.position {
            SeelenWegSide::Left => {
                self.theoretical_rect.right = self.theoretical_rect.left + total_size;
                hidden_rect.right = hidden_rect.left + 1;
            }
            SeelenWegSide::Right => {
                self.theoretical_rect.left = self.theoretical_rect.right - total_size;
                hidden_rect.left = hidden_rect.right - 1;
            }
            SeelenWegSide::Top => {
                self.theoretical_rect.bottom = self.theoretical_rect.top + total_size;
                hidden_rect.bottom = hidden_rect.top + 1;
            }
            SeelenWegSide::Bottom => {
                self.theoretical_rect.top = self.theoretical_rect.bottom - total_size;
                hidden_rect.top = hidden_rect.bottom - 1;
            }
        }

        let mut abd = AppBarData::from_handle(hwnd);
        match settings.hide_mode {
            HideMode::Never => {
                abd.set_edge(settings.position.into());
                abd.set_rect(self.theoretical_rect);
                abd.register_as_new_bar();
            }
            _ => abd.unregister_bar(),
        };

        // pre set position for resize in case of multiples dpi
        WindowsApi::move_window(hwnd, &rc_work)?;
        WindowsApi::set_position(hwnd, None, &rc_work, SWP_NOACTIVATE)?;
        Ok(())
    }
}

impl SeelenWeg {
    pub const TITLE: &'static str = "SeelenWeg";
    const TARGET: &'static str = "@seelen/weg";

    fn create_window(postfix: &str) -> Result<WebviewWindow> {
        let manager = get_app_handle();

        let label = format!("{}?monitor={}", Self::TARGET, postfix);
        log::info!("Creating {}", label);
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&label);

        let window = tauri::WebviewWindowBuilder::new(
            manager,
            label,
            tauri::WebviewUrl::App("seelenweg/index.html".into()),
        )
        .title(Self::TITLE)
        .minimizable(false)
        .maximizable(false)
        .closable(false)
        .resizable(false)
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()?;

        window.set_ignore_cursor_events(true)?;
        let label = window.label().to_string();
        window.listen("request-all-open-apps", move |_| {
            let handler = get_app_handle();
            let apps = &*trace_lock!(OPEN_APPS);
            log_error!(handler.emit_to(&label, "add-multiple-open-apps", apps));
        });
        Ok(window)
    }

    pub fn hide_taskbar() -> JoinHandle<()> {
        std::thread::spawn(move || match get_taskbars_handles() {
            Ok(handles) => {
                let mut attempts = 0;
                while attempts < 10 && FULL_STATE.load().is_weg_enabled() {
                    for handle in &handles {
                        let app_bar = AppBarData::from_handle(*handle);
                        trace_lock!(TASKBAR_STATE_ON_INIT)
                            .insert(handle.0 as isize, app_bar.state());
                        app_bar.set_state(AppBarDataState::AutoHide);
                        let _ = WindowsApi::show_window(*handle, SW_HIDE);
                    }
                    attempts += 1;
                    sleep_millis(50);
                }
            }
            Err(err) => log::error!("Failed to get taskbars handles: {:?}", err),
        })
    }

    pub fn restore_taskbar() -> Result<()> {
        for hwnd in get_taskbars_handles()? {
            AppBarData::from_handle(hwnd).set_state(
                *trace_lock!(TASKBAR_STATE_ON_INIT)
                    .get(&(hwnd.0 as isize))
                    .unwrap_or(&AppBarDataState::AlwaysOnTop),
            );
            WindowsApi::show_window(hwnd, SW_SHOWNORMAL)?;
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref TASKBAR_STATE_ON_INIT: Mutex<HashMap<isize, AppBarDataState>> =
        Mutex::new(HashMap::new());
    pub static ref TASKBAR_CLASS: Vec<&'static str> =
        Vec::from(["Shell_TrayWnd", "Shell_SecondaryTrayWnd",]);
}

pub fn get_taskbars_handles() -> Result<Vec<HWND>> {
    let mut founds = Vec::new();
    WindowEnumerator::new().for_each(|hwnd| {
        let class = WindowsApi::get_class(hwnd).unwrap_or_default();
        if TASKBAR_CLASS.contains(&class.as_str()) {
            founds.push(hwnd);
        }
    })?;
    Ok(founds)
}
