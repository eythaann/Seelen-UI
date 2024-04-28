pub mod handler;
pub mod icon_extractor;
pub mod hook;

use std::{env::temp_dir, path::PathBuf};

use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use serde::Serialize;
use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use win_screenshot::capture::capture_window;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, RECT},
        UI::{
            Shell::{SHAppBarMessage, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA},
            WindowsAndMessaging::{
                FindWindowW, GetParent, GetWindowLongW, ShowWindow, GWL_EXSTYLE, SHOW_WINDOW_CMD,
                SW_HIDE, SW_SHOWNORMAL, WINDOW_EX_STYLE, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
                WS_EX_TOOLWINDOW,
            },
        },
    },
};

use crate::{
    error_handler::Result,
    utils::{are_overlaped, filename_from_path},
    windows_api::WindowsApi,
};

use self::icon_extractor::get_images_from_exe;

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

pub struct SeelenWeg {
    handle: AppHandle<Wry>,
    apps: Vec<SeelenWegApp>,
    hitbox_handle: isize,
    window_handle: isize,
    overlaped: bool,
    last_hitbox_rect: Option<RECT>,
}

impl SeelenWeg {
    const TARGET: &'static str = "seelenweg";
    const TARGET_HITBOX: &'static str = "seelenweg-hitbox";

    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self {
            handle,
            apps: Vec::new(),
            hitbox_handle: 0,
            window_handle: 0,
            overlaped: false,
            last_hitbox_rect: None,
        }
    }

    fn create_window(&self) -> Result<(WebviewWindow, WebviewWindow)> {
        let hitbox = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            &self.handle,
            Self::TARGET_HITBOX,
            tauri::WebviewUrl::App("seelenweg-hitbox/index.html".into()),
        )
        .title("SeelenWeg Hitbox")
        .inner_size(0.0, 0.0)
        .position(0.0, 0.0)
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
            &self.handle,
            Self::TARGET,
            tauri::WebviewUrl::App("seelenweg/index.html".into()),
        )
        .title("SeelenWeg")
        .position(0.0, 0.0)
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

        Ok((window, hitbox))
    }

    pub fn set_active_window(&self, hwnd: HWND) -> Result<()> {
        if WindowsApi::get_window_text(hwnd) == "Task Switching" {
            return Ok(());
        }
        self.handle
            .emit_to(Self::TARGET, "set-focused-handle", hwnd.0)?;
        Ok(())
    }

    fn load_uwp_apps(&self) -> Result<()> {
        let pwsh_script = include_str!("load_uwp_apps.ps1");
        let pwsh_script_path = temp_dir().join("load_uwp_apps.ps1");
        std::fs::write(&pwsh_script_path, pwsh_script).expect("Failed to write temp script file");

        tauri::async_runtime::block_on(async move {
            let result = self
                .handle
                .shell()
                .command("powershell")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-NoProfile",
                    "-File",
                    &pwsh_script_path.to_string_lossy(),
                    "-SavePath",
                    &self
                        .generated_files_path()
                        .join("uwp_manifests.json")
                        .to_string_lossy()
                        .trim_start_matches("\\\\?\\"),
                ])
                .status()
                .await;

            match result {
                Ok(status) => {
                    log::trace!(
                        "load_uwp_apps exit code: {}",
                        status.code().unwrap_or_default()
                    );
                }
                Err(err) => log::error!("load_uwp_apps Failed to wait for process: {}", err),
            };
        });

        Ok(())
    }

    // TODO(eythan) remove this, add to new.
    pub fn start(&mut self) -> Result<()> {
        log::trace!("Starting SeelenWeg");

        self.hide_taskbar(true);
        self.load_uwp_apps()?;
        let (window, hitbox) = self.create_window()?;
        self.hitbox_handle = hitbox.hwnd()?.0;
        self.window_handle = window.hwnd()?.0;
        Self::enum_opened_windows();
        Ok(())
    }

    pub fn stop(&self) {
        self.hide_taskbar(false);
    }

    pub fn generated_files_path(&self) -> PathBuf {
        self.handle
            .path()
            .app_data_dir()
            .expect("Failed to resolve gen path")
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
        let gen_icons_paths = self.generated_files_path().join("icons");
        if !gen_icons_paths.exists() {
            std::fs::create_dir_all(&gen_icons_paths)?;
        }

        let filename = filename_from_path(exe_path);
        let icon_path = gen_icons_paths.join(filename.replace(".exe", ".png"));
        let icon_path_uwp = gen_icons_paths.join(filename.replace(".exe", "_uwp.png"));

        if !icon_path.exists() && !icon_path_uwp.exists() {
            let images = get_images_from_exe(exe_path);
            if let Ok(images) = images {
                // icon on index 0 always is the app showed icon
                if let Some(icon) = images.get(0) {
                    icon.save(&icon_path).expect("Failed to save icon");
                }
            }
        }

        let mut icon_to_save = icon_path;
        if icon_path_uwp.exists() {
            icon_to_save = icon_path_uwp;
        }

        Ok(icon_to_save
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string())
    }

    pub fn hide_taskbar(&self, hide: bool) {
        let lparam: LPARAM;
        let cmdshow: SHOW_WINDOW_CMD;
        if hide {
            lparam = LPARAM(ABS_AUTOHIDE as isize);
            cmdshow = SW_HIDE;
        } else {
            lparam = LPARAM(ABS_ALWAYSONTOP as isize);
            cmdshow = SW_SHOWNORMAL;
        }

        let name: Vec<u16> = format!("Shell_TrayWnd\0").encode_utf16().collect();
        let mut ap_bar: APPBARDATA = unsafe { std::mem::zeroed() };

        ap_bar.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
        ap_bar.hWnd = unsafe { FindWindowW(PCWSTR(name.as_ptr()), PCWSTR::null()) };

        if ap_bar.hWnd.0 != 0 {
            ap_bar.lParam = lparam;
            unsafe {
                ShowWindow(ap_bar.hWnd, cmdshow);
                SHAppBarMessage(ABM_SETSTATE, &mut ap_bar as *mut APPBARDATA);
            }
        }
    }

    pub fn update_ui(&self) {
        self.handle
            .emit_to(Self::TARGET, "add-open-app-many", self.apps.clone())
            .expect("Failed to emit");
    }

    pub fn contains_app(&self, hwnd: HWND) -> bool {
        self.apps.iter().any(|app| app.hwnd == hwnd.0)
    }

    pub fn update_app(&mut self, hwnd: HWND) {
        let app = self.apps.iter_mut().find(|app| app.hwnd == hwnd.0);
        if let Some(app) = app {
            app.title = WindowsApi::get_window_text(hwnd);
            self.handle
                .emit_to(Self::TARGET, "update-open-app-info", app.clone())
                .expect("Failed to emit");
        }
    }

    pub fn replace_hwnd(&mut self, old: HWND, new: HWND) {
        let app = self
            .apps
            .iter_mut()
            .find(|app| app.hwnd == old.0)
            .expect("Failed to find app");
        app.hwnd = new.0;
        self.handle
            .emit_to(Self::TARGET, "replace-open-app", app.clone())
            .expect("Failed to emit");
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) {
        if self.contains_app(hwnd) {
            return;
        }

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

        self.handle
            .emit_to(Self::TARGET, "add-open-app", app.clone())
            .expect("Failed to emit");

        self.apps.push(app);
    }

    pub fn remove_hwnd(&mut self, hwnd: HWND) {
        self.apps.retain(|app| app.hwnd != hwnd.0);
        self.handle
            .emit_to(Self::TARGET, "remove-open-app", hwnd.0)
            .expect("Failed to emit");
    }

    pub fn is_overlapping(&self, hwnd: HWND) -> bool {
        let rect = WindowsApi::get_window_rect(hwnd);
        let hitbox_rect = self
            .last_hitbox_rect
            .unwrap_or_else(|| WindowsApi::get_window_rect(HWND(self.hitbox_handle)));
        are_overlaped(&hitbox_rect, &rect)
    }

    pub fn update_status_if_needed(&mut self, hwnd: HWND) -> Result<()> {
        if WindowsApi::is_iconic(hwnd)
            || !WindowsApi::is_window_visible(hwnd)
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
            Some(WindowsApi::get_window_rect(HWND(self.hitbox_handle)))
        } else {
            None
        };

        self.handle
            .emit_to(Self::TARGET, "set-auto-hide", self.overlaped)?;
        Ok(())
    }

    pub fn is_real_window(hwnd: HWND) -> bool {
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
            || exe_path.ends_with("ApplicationFrameHost.exe")
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
