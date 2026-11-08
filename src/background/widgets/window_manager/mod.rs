pub mod cli;
pub mod handler;
pub mod hook;
pub mod state_v2;

use seelen_core::{handlers::SeelenEvent, state::AppExtraFlag};

pub struct WindowManagerV2;
use crate::{app::emit_to_webviews, error::Result, platform::window::Window};

impl WindowManagerV2 {
    fn should_be_managed(window: &Window) -> bool {
        Self::should_be_managed_impl(window, true)
    }

    /// Same as [Self::should_be_managed] but ignoring the minimized state of the window.
    ///
    /// Used when building the initial layout on startup: windows on non-active workspaces are
    /// minimized by the virtual desktop manager to hide them (see `VirtualDesktopMonitor::hide`),
    /// but they should still occupy their tiled slot in their own workspace layout.
    fn should_be_managed_ignoring_minimized(window: &Window) -> bool {
        Self::should_be_managed_impl(window, false)
    }

    fn should_be_managed_impl(window: &Window, check_minimized: bool) -> bool {
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

        // Ignore windows that are not resizable
        if !window.is_resizable() {
            return false;
        }

        // Top most windows normally are widgets or tools that should not be managed
        if window.is_always_on_top() {
            return false;
        }

        true
    }

    pub fn force_retiling() -> Result<()> {
        emit_to_webviews(SeelenEvent::WMForceRetiling, ());
        Ok(())
    }
}
