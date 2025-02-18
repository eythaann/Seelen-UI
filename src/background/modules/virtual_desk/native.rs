use parking_lot::Mutex;
use winvd::{Desktop, DesktopEvent, DesktopEventThread};

use crate::{error_handler::Result, trace_lock};

use super::{VirtualDesktop, VirtualDesktopEvent, VirtualDesktopManagerTrait, VirtualDesktopTrait};

impl From<&Desktop> for VirtualDesktop {
    fn from(d: &Desktop) -> Self {
        VirtualDesktop::Native(NativeVirtualDesktop(*d))
    }
}

impl From<Desktop> for VirtualDesktop {
    fn from(d: Desktop) -> Self {
        VirtualDesktop::Native(NativeVirtualDesktop(d))
    }
}

#[derive(Debug, Clone)]
pub struct NativeVirtualDesktop(winvd::Desktop);

impl VirtualDesktopTrait for NativeVirtualDesktop {
    fn id(&self) -> String {
        format!(
            "{:?}",
            self.0.get_id().expect("Failed to get native desktop id")
        )
    }

    fn name(&self) -> Option<String> {
        self.0.get_name().ok()
    }
}

#[derive(Debug)]
pub struct NativeVirtualDesktopManager {
    event_thread: Mutex<Option<DesktopEventThread>>,
}

impl NativeVirtualDesktopManager {
    pub fn new() -> Self {
        Self {
            event_thread: Mutex::new(None),
        }
    }
}

impl VirtualDesktopManagerTrait for NativeVirtualDesktopManager {
    fn uses_cloak(&self) -> bool {
        true
    }

    fn create_desktop(&self) -> Result<()> {
        winvd::create_desktop()?;
        Ok(())
    }

    fn get(&self, idx: usize) -> Result<Option<VirtualDesktop>> {
        let desktop = winvd::get_desktops()?.get(idx).map(|d| d.into());
        Ok(desktop)
    }

    fn get_by_window(&self, _hwnd: isize) -> Result<VirtualDesktop> {
        // Ok(winvd::get_desktop_by_window(HWND(window as _))?.into())
        Err("Not implemented".into())
    }

    fn get_all(&self) -> Result<Vec<VirtualDesktop>> {
        Ok(winvd::get_desktops()?
            .into_iter()
            .map(|d| d.into())
            .collect())
    }

    fn get_current(&self) -> Result<VirtualDesktop> {
        Ok(winvd::get_current_desktop()?.into())
    }

    fn get_current_idx(&self) -> Result<usize> {
        Ok(winvd::get_current_desktop()?.get_index()? as usize)
    }

    fn switch_to(&self, idx: usize) -> Result<()> {
        winvd::switch_desktop(idx as u32)?;
        Ok(())
    }

    fn send_to(&self, _idx: usize, _hwnd: isize) -> Result<()> {
        // winvd::move_window_to_desktop(idx as u32, &HWND(hwnd as _))?;
        Err("Not implemented".into())
    }

    fn pin_window(&self, _hwnd: isize) -> Result<()> {
        // winvd::pin_window(HWND(hwnd as _))?;
        Err("Not implemented".into())
    }

    fn unpin_window(&self, _hwnd: isize) -> Result<()> {
        // winvd::unpin_window(HWND(hwnd as _))?;
        Err("Not implemented".into())
    }

    fn is_pinned_window(&self, _hwnd: isize) -> Result<bool> {
        // Ok(winvd::is_pinned_window(HWND(hwnd as _))?)
        Err("Not implemented".into())
    }

    fn listen_events(&self, sender: std::sync::mpsc::Sender<VirtualDesktopEvent>) -> Result<()> {
        *trace_lock!(self.event_thread) = Some(winvd::listen_desktop_events(sender)?);
        Ok(())
    }
}

impl From<DesktopEvent> for VirtualDesktopEvent {
    fn from(event: DesktopEvent) -> Self {
        match event {
            DesktopEvent::DesktopCreated(desktop) => {
                VirtualDesktopEvent::DesktopCreated(desktop.into())
            }
            DesktopEvent::DesktopDestroyed {
                destroyed,
                fallback,
            } => VirtualDesktopEvent::DesktopDestroyed {
                destroyed: destroyed.into(),
                fallback: fallback.into(),
            },
            DesktopEvent::DesktopChanged { new, old } => VirtualDesktopEvent::DesktopChanged {
                new: new.into(),
                old: old.into(),
            },
            DesktopEvent::DesktopNameChanged(desktop, name) => {
                VirtualDesktopEvent::DesktopNameChanged(desktop.into(), name)
            }
            DesktopEvent::DesktopWallpaperChanged(desktop, path) => {
                VirtualDesktopEvent::DesktopWallpaperChanged(desktop.into(), path)
            }
            DesktopEvent::DesktopMoved {
                desktop,
                old_index,
                new_index,
            } => VirtualDesktopEvent::DesktopMoved {
                desktop: desktop.into(),
                old_index: old_index as usize,
                new_index: new_index as usize,
            },
            DesktopEvent::WindowChanged(window) => {
                VirtualDesktopEvent::WindowChanged(window.0 as _)
            }
        }
    }
}
