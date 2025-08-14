use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::rect::Rect;
use std::sync::Arc;

use crate::{
    error::Result,
    trace_lock,
    virtual_desktops::events::VirtualDesktopEvent,
    windows_api::window::event::WinEvent,
    windows_api::{monitor::Monitor, window::Window},
};

use super::{cli::Axis, state::WM_STATE, WindowManagerV2};

lazy_static! {
    static ref SystemMoveSizeStartRect: Arc<Mutex<Rect>> = Arc::new(Mutex::new(Rect::default()));
    static ref SystemMoveSizeStartMonitor: Arc<Mutex<Monitor>> =
        Arc::new(Mutex::new(Monitor::from(0)));
}

impl WindowManagerV2 {
    pub fn process_vd_event(event: &VirtualDesktopEvent) -> Result<()> {
        match event {
            VirtualDesktopEvent::DesktopChanged { monitor, workspace } => {
                // TODO: implement
                // Self::discard_reservation()?;
                Self::workspace_changed(monitor, workspace)?;
            }
            VirtualDesktopEvent::WindowChanged { window, .. } => {
                let window = &Window::from(*window);
                if Self::is_managed(window) {
                    log::trace!("window changed: {window:?}");
                    Self::remove(window)?;
                    Self::add(window)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn system_move_size_end(window: &Window) -> Result<()> {
        if !Self::is_managed(window) {
            return Ok(());
        }

        if *trace_lock!(SystemMoveSizeStartMonitor) != window.monitor() {
            log::trace!("window moved of monitor");
            Self::remove(window)?;
            Self::add(window)?;
            Self::set_overlay_visibility(true)?;
            return Ok(());
        }

        let initial_rect = trace_lock!(SystemMoveSizeStartRect);
        let initial_width = (initial_rect.right - initial_rect.left) as f32;
        let initial_height = (initial_rect.bottom - initial_rect.top) as f32;

        let rect = window.inner_rect()?;
        let new_width = (rect.right - rect.left) as f32;
        let new_height = (rect.bottom - rect.top) as f32;

        if initial_width != new_width {
            let percentage_diff = (new_width - initial_width) / initial_width * 100.0;
            let axis = if rect.left == initial_rect.left {
                Axis::Right
            } else {
                Axis::Left
            };
            log::trace!("window width changed by: {percentage_diff}%");
            let state = trace_lock!(WM_STATE);
            let (m, w) = state.update_size(window, axis, percentage_diff, true)?;
            Self::render_workspace(&m.id, w)?;
        }

        if initial_height != new_height {
            let percentage_diff = (new_height - initial_height) / initial_height * 100.0;
            let axis = if rect.top == initial_rect.top {
                Axis::Bottom
            } else {
                Axis::Top
            };
            log::trace!("window height changed by: {percentage_diff}%");
            let state = trace_lock!(WM_STATE);
            let (m, w) = state.update_size(window, axis, percentage_diff, true)?;
            Self::render_workspace(&m.id, w)?;
        }

        Self::force_retiling()?;
        Self::set_overlay_visibility(true)?;
        Ok(())
    }

    pub fn process_win_event(event: WinEvent, window: &Window) -> Result<()> {
        match event {
            WinEvent::SystemMoveSizeStart => {
                if Self::is_managed(window) {
                    Self::set_overlay_visibility(false)?;
                    *trace_lock!(SystemMoveSizeStartRect) = window.inner_rect()?;
                    *trace_lock!(SystemMoveSizeStartMonitor) = window.monitor();
                }
            }
            WinEvent::SystemMoveSizeEnd => Self::system_move_size_end(window)?,
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
                if window.is_focused()
                    && !Self::is_managed(window)
                    && Self::should_be_managed(window.hwnd())
                {
                    Self::add(window)?;
                    Self::set_overlay_visibility(true)?;
                }
            }
            WinEvent::SyntheticMaximizeStart => {
                // todo make this by monitor
                Self::set_overlay_visibility(false)?;
            }
            WinEvent::SyntheticMaximizeEnd => {
                // todo make this by monitor
                Self::set_overlay_visibility(true)?;
            }
            WinEvent::SyntheticFullscreenStart => Self::set_overlay_visibility(false)?,
            WinEvent::SyntheticFullscreenEnd => Self::set_overlay_visibility(true)?,
            _ => {}
        };
        Ok(())
    }
}
