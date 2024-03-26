pub mod handler;

use std::env::temp_dir;

use serde::Serialize;
use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{BOOL, HWND, LPARAM},
        UI::{
            Shell::{SHAppBarMessage, ABM_SETSTATE, ABS_ALWAYSONTOP, ABS_AUTOHIDE, APPBARDATA},
            WindowsAndMessaging::{
                EnumWindows, FindWindowW, GetParent, GetWindowLongW, ShowWindow, GWL_EXSTYLE,
                GWL_STYLE, SHOW_WINDOW_CMD, SW_HIDE, SW_SHOWNORMAL, WINDOW_EX_STYLE, WINDOW_STYLE,
                WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_POPUP,
            },
        },
    },
};

use crate::{error_handler::Result, seelen::SEELEN, windows_api::WindowsApi};

#[derive(Debug, Serialize, Clone)]
pub struct SeelenWegApp {
    hwnd: isize,
    exe: String,
    title: String,
    icon: String,
}

pub struct SeelenWeg {
    handle: AppHandle<Wry>,
    opened_apps: Vec<SeelenWegApp>,
}

impl SeelenWeg {
    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self {
            handle,
            opened_apps: Vec::new(),
        }
    }

    fn create_window(&self) -> Result<WebviewWindow> {
        tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            &self.handle,
            "seelenweg-hitbox",
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
            "seelenweg",
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
            SEELEN.lock().mut_weg().add_hwnd(hwnd);
        }
        true.into()
    }

    fn enum_opened_apps(&mut self) {
        unsafe {
            EnumWindows(Some(Self::enum_opened_apps_proc), LPARAM(0))
                .expect("Failed to enum windows");
        };
    }

    pub fn start(&mut self) -> Result<()> {
        log::trace!("Starting SeelenWeg");

        self.auto_hide_taskbar(true);
        self.create_window()?;
        self.enum_opened_apps();
        Ok(())
    }

    pub fn stop(&self) {
        self.auto_hide_taskbar(false);
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
        let pwsh_script = include_str!("extract_icon.ps1");
        let pwsh_script_path = temp_dir().join("extract_icon.ps1");

        if !pwsh_script_path.exists() {
            std::fs::write(&pwsh_script_path, pwsh_script)
                .expect("Failed to write temp script file");
        }

        let handle = &self.handle;
        let gen_icons_paths = handle
            .path()
            .resolve("gen/icons", BaseDirectory::Resource)?;

        handle
            .shell()
            .command("powershell")
            .args([
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-File",
                &pwsh_script_path.to_string_lossy(),
                "-exe",
                exe_path,
                "-ExtractDir",
                &gen_icons_paths
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\"),
            ])
            .spawn()
            .expect("Failed to spawn icon extraction script");

        std::fs::remove_file(pwsh_script_path)?;

        let ico_path = gen_icons_paths
            .join(
                exe_path
                    .replace(".exe", ".png")
                    .split("\\")
                    .last()
                    .unwrap_or_default(),
            )
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string();

        Ok(ico_path)
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
            .emit("update-store-apps", self.opened_apps.clone())
            .expect("Failed to emit");
    }

    pub fn add_hwnd(&mut self, hwnd: HWND) {
        if self.opened_apps.iter().any(|app| app.hwnd == hwnd.0) {
            return;
        }

        let exe_path = WindowsApi::exe_path(hwnd).unwrap_or_default();
        let mut icon_path = self.missing_icon();
        if exe_path != "" {
            icon_path = self.extract_icon(&exe_path).unwrap_or(icon_path);
        }

        self.opened_apps.push(SeelenWegApp {
            hwnd: hwnd.0,
            exe: exe_path,
            title: WindowsApi::get_window_text(hwnd),
            icon: icon_path,
        });
        self.update_ui();
    }

    pub fn remove_hwnd(&mut self, hwnd: HWND) {
        self.opened_apps.retain(|app| app.hwnd != hwnd.0);
        self.update_ui();
    }

    pub fn should_handle_hwnd(hwnd: HWND) -> bool {
        if WindowsApi::is_window_visible(hwnd) {
            unsafe {
                let parent = GetParent(hwnd);
                if parent.0 == 0 {
                    let ex_style = WINDOW_EX_STYLE(GetWindowLongW(hwnd, GWL_EXSTYLE) as u32);
                    let style = WINDOW_STYLE(GetWindowLongW(hwnd, GWL_STYLE) as u32);

                    let is_popup = style.contains(WS_POPUP); // Todo(eythan) ensure this is correct
                    let is_tool = ex_style.contains(WS_EX_TOOLWINDOW);
                    let is_app = ex_style.contains(WS_EX_APPWINDOW);
                    return (!is_popup && !is_tool) || is_app;
                }
            }
        }
        false
    }
}
