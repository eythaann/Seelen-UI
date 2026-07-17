pub mod cli;
pub mod handler;
pub mod hook;
pub mod state_v2;

use seelen_core::{handlers::SeelenEvent, state::AppExtraFlag};

pub struct WindowManagerV2;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{WS_EX_TOPMOST, WS_SIZEBOX},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    windows_api::{window::Window, WindowsApi},
};

impl WindowManagerV2 {
    fn should_be_managed(hwnd: HWND) -> bool {
        Self::should_be_managed_impl(hwnd, true)
    }

    /// Same as [Self::should_be_managed] but ignoring the minimized state of the window.
    ///
    /// Used when building the initial layout on startup: windows on non-active workspaces are
    /// minimized by the virtual desktop manager to hide them (see `VirtualDesktopMonitor::hide`),
    /// but they should still occupy their tiled slot in their own workspace layout.
    fn should_be_managed_ignoring_minimized(hwnd: HWND) -> bool {
        Self::should_be_managed_impl(hwnd, false)
    }

    fn should_be_managed_impl(hwnd: HWND, check_minimized: bool) -> bool {
        let window = Window::from(hwnd);
        if !window.is_interactable_and_not_hidden() || (check_minimized && window.is_minimized()) {
            return false;
        }

        if let Ok(Some(config)) = window.get_app_config() {
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
        emit_to_webviews(SeelenEvent::WMForceRetiling, ());
        Ok(())
    }
}
