pub mod handler;
pub mod icon_extractor;

use std::{env::temp_dir, path::PathBuf, process::Command};

use image::{DynamicImage, RgbaImage};
use lazy_static::lazy_static;
use serde::Serialize;
use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewWindow, Wry};
use win_screenshot::capture::capture_window;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, HWND, LPARAM},
        UI::{
            Shell::{SHAppBarMessage, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA},
            WindowsAndMessaging::{
                EnumWindows, FindWindowW, GetParent, GetWindowLongW, ShowWindow, GWL_EXSTYLE,
                SHOW_WINDOW_CMD, SW_HIDE, SW_SHOWNORMAL, WINDOW_EX_STYLE, WS_EX_APPWINDOW,
                WS_EX_TOOLWINDOW,
            },
        },
    },
};

use crate::{error_handler::Result, seelen::SEELEN, windows_api::WindowsApi};

use self::icon_extractor::get_images_from_exe;

lazy_static! {
    static ref BLACK_LIST: Vec<&'static str> = Vec::from(["", "SeelenWeg", "SeelenWeg Hitbox",]);
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
}

impl SeelenWeg {
    const TARGET: &'static str = "seelenweg";
    const TARGET_HITBOX: &'static str = "seelenweg-hitbox";

    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self {
            handle,
            apps: Vec::new(),
        }
    }

    fn create_window(&self) -> Result<WebviewWindow> {
        tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            &self.handle,
            Self::TARGET_HITBOX,
            tauri::WebviewUrl::App("seelenweg-hitbox/index.html".into()),
        )
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .title("SeelenWeg Hitbox")
        .visible(true)
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
        .position(0.0, 0.0)
        .maximizable(false)
        .minimizable(false)
        .resizable(false)
        .title("SeelenWeg")
        .visible(true)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()?;

        window.set_ignore_cursor_events(true)?;

        Ok(window)
    }

    unsafe extern "system" fn enum_opened_apps_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        if SeelenWeg::should_handle_hwnd(hwnd) {
            SEELEN.lock().weg_mut().add_hwnd(hwnd);
        }
        true.into()
    }

    fn enum_opened_apps(&mut self) {
        unsafe {
            EnumWindows(Some(Self::enum_opened_apps_proc), LPARAM(0))
                .expect("Failed to enum windows");
        };
    }

    fn load_uwp_apps(&self) -> Result<()> {
        let pwsh_script = include_str!("load_uwp_apps.ps1");
        let pwsh_script_path = temp_dir().join("load_uwp_apps.ps1");
        std::fs::write(&pwsh_script_path, pwsh_script).expect("Failed to write temp script file");
        let mut child = Command::new("powershell")
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
            .spawn()
            .expect("Failed to spawn uwp load script");

        match child.wait() {
            Ok(status) => {
                log::trace!(
                    "load_uwp_apps exit code: {}",
                    status.code().unwrap_or_default()
                );
            }
            Err(err) => log::error!("load_uwp_apps Failed to wait for process: {}", err),
        };

        std::fs::remove_file(pwsh_script_path)?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        log::trace!("Starting SeelenWeg");

        self.auto_hide_taskbar(true);
        self.enum_opened_apps();
        self.load_uwp_apps()?;
        self.create_window()?;
        Ok(())
    }

    pub fn stop(&self) {
        self.auto_hide_taskbar(false);
    }

    pub fn generated_files_path(&self) -> PathBuf {
        self.handle
            .path()
            .resolve("gen", BaseDirectory::Resource)
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

        let icon_path = gen_icons_paths.join(
            exe_path
                .replace(".exe", ".png")
                .split("\\")
                .last()
                .unwrap_or_default(),
        );

        if !icon_path.exists() {
            let images = get_images_from_exe(exe_path);
            if let Ok(images) = images {
                // icon on index 0 always is the app showed icon
                if let Some(icon) = images.get(0) {
                    icon.save(&icon_path).expect("Failed to save icon");
                }
            }
        }

        Ok(icon_path
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string())
    }

    fn auto_hide_taskbar(&self, hide: bool) {
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
            .emit_to(Self::TARGET, "update-store-apps", self.apps.clone())
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
                .emit_to(Self::TARGET, "update-open-app", app.clone())
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

    pub fn should_handle_hwnd(hwnd: HWND) -> bool {
        if !WindowsApi::is_window_visible(hwnd) {
            return false;
        }

        let parent = unsafe { GetParent(hwnd) };
        if parent.0 != 0 {
            return false;
        }

        let ex_style = WINDOW_EX_STYLE(unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32);
        let is_tool = ex_style.contains(WS_EX_TOOLWINDOW);
        let is_app = ex_style.contains(WS_EX_APPWINDOW);

        if is_tool && !is_app {
            return false;
        }

        let exe_path = WindowsApi::exe_path(hwnd).unwrap_or_default();
        if exe_path.starts_with("C:\\Windows\\SystemApps")
            || exe_path.ends_with("ApplicationFrameHost.exe")
        {
            return false;
        }

        let title = WindowsApi::get_window_text(hwnd);
        !BLACK_LIST.contains(&title.as_str())
    }

    pub fn capture_window(hwnd: HWND) -> Option<DynamicImage> {
        capture_window(hwnd.0).ok().map(|buf| {
            let image = RgbaImage::from_raw(buf.width, buf.height, buf.pixels).unwrap_or_default();
            DynamicImage::ImageRgba8(image)
        })
    }
}
