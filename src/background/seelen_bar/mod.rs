pub mod cli;

use tauri::{AppHandle, WebviewWindow, Wry};

use crate::error_handler::Result;

pub struct FancyToolbar {
    #[allow(dead_code)]
    handle: AppHandle<Wry>,
    window: WebviewWindow,
    hitbox_window: WebviewWindow,
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
        }
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

        Ok((window, hitbox))
    }
}
