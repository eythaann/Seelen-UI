pub mod cli;
pub mod handler;
pub mod hook;
pub mod instance;
pub mod state;

use instance::WindowManagerV2;
use seelen_core::{handlers::SeelenEvent, state::AppExtraFlag};
use tauri::Emitter;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{WS_EX_TOPMOST, WS_SIZEBOX},
};

use crate::{
    app::get_app_handle,
    error::Result,
    state::application::FULL_STATE,
    windows_api::{window::Window, WindowsApi},
};

impl WindowManagerV2 {
    fn should_be_managed(hwnd: HWND) -> bool {
        let window = Window::from(hwnd);
        if !window.is_interactable_and_not_hidden() {
            return false;
        }

        if let Ok(Some(config)) = FULL_STATE.load().get_app_config_by_window(hwnd) {
            if config.options.contains(&AppExtraFlag::VdPinned) {
                return false;
            }

            if config.options.contains(&AppExtraFlag::WmForce) {
                return true;
            }

            if config.options.contains(&AppExtraFlag::WmUnmanage) {
                return false;
            }
        }

        let styles = WindowsApi::get_styles(hwnd);
        // Ignore windows that are not resizable
        if !styles.contains(WS_SIZEBOX) {
            return false;
        }

        let ex_styles = WindowsApi::get_ex_styles(hwnd);
        // Top most windows normally are widgets or tools that should not be managed
        if ex_styles.contains(WS_EX_TOPMOST) {
            return false;
        }

        true
    }

    pub fn force_retiling() -> Result<()> {
        get_app_handle().emit(SeelenEvent::WMForceRetiling, ())?;
        Ok(())
    }
}
