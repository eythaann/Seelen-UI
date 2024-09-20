use crate::{
    error_handler::Result, modules::virtual_desk::VirtualDesktopEvent, windows_api::window::Window,
    winevent::WinEvent,
};

use super::WindowManagerV2;

impl WindowManagerV2 {
    pub fn process_vd_event(event: &VirtualDesktopEvent) -> Result<()> {
        match event {
            VirtualDesktopEvent::DesktopChanged { new, old: _ } => {
                // Self::discard_reservation()?;
                Self::workspace_changed(new)?;
            }
            VirtualDesktopEvent::WindowChanged(window) => {
                log::trace!("window changed: {:?}", window);
                let window = &Window::from(*window);
                if Self::is_managed(window) {
                    Self::remove(window)?;
                    Self::add(window)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn process_win_event(event: WinEvent, window: &Window) -> Result<()> {
        match event {
            WinEvent::SystemMoveSizeStart => {
                if Self::is_managed(window) {
                    Self::set_overlay_visibility(false)?;
                }
            }
            WinEvent::SystemMoveSizeEnd => {
                if Self::is_managed(window) {
                    Self::force_retiling()?;
                    Self::set_overlay_visibility(true)?;
                }
            }
            WinEvent::ObjectCreate | WinEvent::ObjectShow | WinEvent::SystemMinimizeEnd => {
                if !Self::is_managed(window) && Self::should_be_managed(window.hwnd()) {
                    Self::add(window)?;
                    Self::set_overlay_visibility(true)?;
                }
            }
            WinEvent::ObjectDestroy | WinEvent::ObjectHide | WinEvent::SystemMinimizeStart => {
                if Self::is_managed(window) {
                    Self::remove(window)?;
                }
            }
            WinEvent::ObjectFocus | WinEvent::SystemForeground => {
                Self::set_active_window(window)?;
                Self::set_overlay_visibility(Self::is_managed(window))?;
            }
            // apps like firefox doesn't launch ObjectCreate
            WinEvent::ObjectNameChange => {
                if window.is_foreground()
                    && !Self::is_managed(window)
                    && Self::should_be_managed(window.hwnd())
                {
                    Self::add(window)?;
                    Self::set_overlay_visibility(true)?;
                }
            }
            WinEvent::ObjectLocationChange => {
                if window.is_foreground() && window.is_maximized() {
                    Self::set_overlay_visibility(false)?;
                }
            }
            WinEvent::SyntheticFullscreenStart(_) => Self::set_overlay_visibility(false)?,
            WinEvent::SyntheticFullscreenEnd(_) => Self::set_overlay_visibility(true)?,
            _ => {}
        };
        Ok(())
    }
}
