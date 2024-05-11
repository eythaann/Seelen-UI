pub mod cli;
pub mod hook;

use crate::{
    error_handler::{log_if_error, Result},
    seelen::SEELEN,
    windows_api::WindowsApi,
};
use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewWindow, Wry};
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Shell::{SHAppBarMessage, ABE_TOP, ABM_NEW, ABM_SETPOS, APPBARDATA},
        WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SWP_ASYNCWINDOWPOS},
    },
};

pub struct FancyToolbar {
    #[allow(dead_code)]
    handle: AppHandle<Wry>,
    window: WebviewWindow,
    hitbox_window: WebviewWindow,
    // -- -- -- --
    last_focus: Option<isize>,
}

#[derive(Serialize, Clone)]
pub struct ActiveApp {
    title: String,
    name: String,
}

impl FancyToolbar {
    pub fn new(handle: AppHandle<Wry>) -> Self {
        log::info!("Creating Fancy Toolbar");

        let (window, hitbox_window) =
            Self::create_window(&handle).expect("Failed to create window");

        Self {
            handle,
            window,
            hitbox_window,
            last_focus: None,
        }
    }

    pub fn focus_changed(&mut self, hwnd: HWND) -> Result<()> {
        let title = WindowsApi::get_window_text(hwnd);

        self.last_focus = Some(hwnd.0);
        self.handle.emit_to(
            Self::TARGET,
            "focus-changed",
            ActiveApp {
                title,
                name: WindowsApi::get_window_display_name(hwnd)
                    .unwrap_or(String::from("Error on App Name")),
            },
        )?;
        Ok(())
    }

    pub fn ensure_hitbox_zorder(&self) -> Result<()> {
        WindowsApi::bring_to(
            HWND(self.hitbox_window.hwnd()?.0),
            HWND(self.window.hwnd()?.0),
        )
    }
}

// statics
impl FancyToolbar {
    const TARGET: &'static str = "fancy-toolbar";
    const TARGET_HITBOX: &'static str = "fancy-toolbar-hitbox";

    fn create_window(manager: &AppHandle<Wry>) -> Result<(WebviewWindow, WebviewWindow)> {
        let hitbox = tauri::WebviewWindowBuilder::<Wry, AppHandle<Wry>>::new(
            manager,
            Self::TARGET_HITBOX,
            tauri::WebviewUrl::App("toolbar-hitbox/index.html".into()),
        )
        .title("Seelen Fancy Toolbar Hitbox")
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
            manager,
            Self::TARGET,
            tauri::WebviewUrl::App("toolbar/index.html".into()),
        )
        .title("Seelen Fancy Toolbar")
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
        window.listen("complete-setup", move |event| unsafe {
            let seelen = SEELEN.lock();
            let ft = seelen.toolbar().unwrap();

            let height: i32 = event.payload().parse().unwrap_or(0);
            let main_hwnd = ft.window.hwnd().expect("Failed to get HWND");

            let mut abd = APPBARDATA::default();
            abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
            abd.hWnd = ft.hitbox_window.hwnd().expect("Failed to get HWND");
            abd.uEdge = ABE_TOP;

            abd.rc.bottom = height;
            abd.rc.right = GetSystemMetrics(SM_CXSCREEN);

            SHAppBarMessage(ABM_NEW, &mut abd);
            SHAppBarMessage(ABM_SETPOS, &mut abd);

            WindowsApi::set_position(abd.hWnd, None, &abd.rc, SWP_ASYNCWINDOWPOS)
                .expect("Failed to set position");

            let mut rect = abd.rc.clone();
            rect.bottom = GetSystemMetrics(SM_CXSCREEN);
            WindowsApi::set_position(main_hwnd, None, &rect, SWP_ASYNCWINDOWPOS)
                .expect("Failed to set position");

            log::info!("Fancy Toolbar setup is completed");
            log_if_error(seelen.handle().emit("toolbar-setup-completed", ()));
            ft.window.unlisten(event.id());
        });

        Ok((window, hitbox))
    }
}
