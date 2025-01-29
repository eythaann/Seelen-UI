mod hook;

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use arc_swap::ArcSwap;
use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};

use crate::{
    error_handler::{AppError, Result},
    hook::HookManager,
    log_error, trace_lock,
    windows_api::window::Window,
    winevent::WinEvent,
};

use super::{VirtualDesktop, VirtualDesktopEvent, VirtualDesktopManagerTrait, VirtualDesktopTrait};
use windows::Win32::UI::WindowsAndMessaging::{SW_FORCEMINIMIZE, SW_MINIMIZE, SW_RESTORE};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeelenWorkspace {
    id: String,
    name: Option<String>,
    wallpaper: Option<PathBuf>,
    #[serde(skip)]
    windows: Vec<isize>,
}

impl From<&SeelenWorkspace> for VirtualDesktop {
    fn from(value: &SeelenWorkspace) -> Self {
        VirtualDesktop::Seelen(value.clone())
    }
}

impl From<SeelenWorkspace> for VirtualDesktop {
    fn from(value: SeelenWorkspace) -> Self {
        VirtualDesktop::Seelen(value)
    }
}

impl VirtualDesktopTrait for SeelenWorkspace {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> Option<String> {
        self.name.clone()
    }
}

impl SeelenWorkspace {
    fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: None,
            wallpaper: None,
            windows: Vec::new(),
        }
    }

    fn remove_window(&mut self, address: isize) {
        let window = Window::from(address);
        self.windows.retain(|w| *w != address);
        HookManager::run_with_async(move |hook_manager| {
            hook_manager.skip(WinEvent::SystemMinimizeStart, window.hwnd());
            log_error!(window.show_window_async(SW_FORCEMINIMIZE))
        });
    }

    fn hide(&self) {
        let win_address = self.windows.clone();
        HookManager::run_with_async(move |hook_manager| {
            for addr in win_address {
                let window = Window::from(addr);
                if window.is_window() {
                    hook_manager.skip(WinEvent::SystemMinimizeStart, window.hwnd());
                    log_error!(window.show_window_async(SW_MINIMIZE));
                }
            }
        });
    }

    fn restore(&self) {
        let win_address = self.windows.clone();
        HookManager::run_with_async(move |hook_manager| {
            for addr in &win_address {
                let window = Window::from(*addr);
                // if is switching by restored window on other workspace it will be already shown
                if window.is_window() && window.is_minimized() {
                    hook_manager.skip(WinEvent::SystemMinimizeEnd, window.hwnd());
                    // use normal show instead async cuz it will keep the order of restoring
                    log_error!(window.show_window(SW_RESTORE));
                }
            }
            if let Some(last) = win_address.last() {
                log_error!(Window::from(*last).focus());
            }
        });
    }
}

#[derive(Debug, Default)]
pub struct SeelenWorkspacesManager {
    current: AtomicUsize,
    sender: ArcSwap<Option<std::sync::mpsc::Sender<VirtualDesktopEvent>>>,
    workspaces: Mutex<Vec<SeelenWorkspace>>,
    pinned: Mutex<Vec<isize>>,
}

pub fn none_err() -> AppError {
    "Seelen Workspace not found".into()
}

impl SeelenWorkspacesManager {
    pub fn new() -> Self {
        let manager = Self {
            current: AtomicUsize::new(0),
            workspaces: Mutex::new(vec![SeelenWorkspace::new()]),
            pinned: Mutex::new(Vec::new()),
            sender: ArcSwap::new(Arc::new(None)),
        };
        log_error!(manager.enumerate());
        manager
    }

    fn current_idx(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }

    fn pinned(&self) -> MutexGuard<'_, Vec<isize>> {
        trace_lock!(self.pinned)
    }

    fn emit(&self, event: VirtualDesktopEvent) -> Result<()> {
        let sender = self.sender.load_full();
        std::thread::spawn(move || {
            if let Some(sender) = sender.as_ref() {
                log_error!(sender.send(event));
            }
        });
        Ok(())
    }

    fn create_many_desktop(&self, count: usize) -> Result<()> {
        log::trace!("Creating {} seelen workspaces", count);
        for _ in 0..count {
            self.create_desktop()?;
        }
        Ok(())
    }
}

impl VirtualDesktopManagerTrait for SeelenWorkspacesManager {
    fn uses_cloak(&self) -> bool {
        false
    }

    fn create_desktop(&self) -> Result<()> {
        log::trace!("Creating new seelen workspace");
        let desk = SeelenWorkspace::new();
        trace_lock!(self.workspaces).push(desk.clone());
        self.emit(VirtualDesktopEvent::DesktopCreated(desk.into()))?;
        Ok(())
    }

    fn get(&self, idx: usize) -> Result<Option<VirtualDesktop>> {
        if let Some(workspace) = trace_lock!(self.workspaces).get_mut(idx) {
            return Ok(Some(workspace.clone().into()));
        }
        Ok(None)
    }

    fn get_by_window(&self, window: isize) -> Result<VirtualDesktop> {
        if self.is_pinned_window(window)? {
            return self.get_current();
        }
        let desk = {
            trace_lock!(self.workspaces)
                .iter()
                .find(|w| w.windows.contains(&window))
                .map(Into::into)
        };
        desk.or_else(|| self.get_current().ok())
            .ok_or_else(none_err)
    }

    fn get_all(&self) -> Result<Vec<VirtualDesktop>> {
        Ok(trace_lock!(self.workspaces)
            .iter()
            .map(Into::into)
            .collect())
    }

    fn get_current(&self) -> Result<VirtualDesktop> {
        if let Some(workspace) = trace_lock!(self.workspaces).get_mut(self.current_idx()) {
            return Ok(workspace.clone().into());
        }
        Err(none_err())
    }

    fn get_current_idx(&self) -> Result<usize> {
        Ok(self.current_idx())
    }

    fn switch_to(&self, idx: usize) -> Result<()> {
        if idx == self.current_idx() {
            return Ok(());
        }

        let len = trace_lock!(self.workspaces).len();
        if idx >= len {
            // temporal until implement a UI to create seelen workspaces
            self.create_many_desktop((idx + 1) - len)?;
            // return Err("Tried to switch to non-existent workspace".into());
        }

        let workspaces = trace_lock!(self.workspaces);
        let old = workspaces.get(self.current_idx()).ok_or_else(none_err)?;
        self.current.store(idx, Ordering::SeqCst);
        let new = workspaces.get(self.current_idx()).ok_or_else(none_err)?;

        old.hide();
        self.emit(VirtualDesktopEvent::DesktopChanged {
            new: new.into(),
            old: old.into(),
        })?;
        new.restore();
        Ok(())
    }

    fn send_to(&self, idx: usize, window: isize) -> Result<()> {
        let len = trace_lock!(self.workspaces).len();
        if idx >= len {
            // temporal until implement a UI to create seelen workspaces
            self.create_many_desktop((idx + 1) - len)?;
            // return Err("Tried to switch to non-existent workspace".into());
        }

        let mut workspaces = trace_lock!(self.workspaces);

        let old_idx = match workspaces.iter().position(|w| w.windows.contains(&window)) {
            Some(idx) => idx,
            None => return Ok(()),
        };

        if old_idx == idx {
            return Ok(());
        }

        {
            let old = workspaces.get_mut(old_idx).ok_or_else(none_err)?;
            old.remove_window(window);
        }
        {
            let new = workspaces.get_mut(idx).ok_or_else(none_err)?;
            new.windows.push(window);
            if self.current_idx() == idx {
                new.restore();
            }
        }

        self.emit(VirtualDesktopEvent::WindowChanged(window))
    }

    fn pin_window(&self, hwnd: isize) -> Result<()> {
        let mut pinned = self.pinned();
        if !pinned.contains(&hwnd) {
            pinned.push(hwnd);
        }
        Ok(())
    }

    fn unpin_window(&self, hwnd: isize) -> Result<()> {
        self.pinned().retain(|w| *w != hwnd);
        Ok(())
    }

    fn is_pinned_window(&self, hwnd: isize) -> Result<bool> {
        Ok(self.pinned().contains(&hwnd))
    }

    fn listen_events(&self, sender: std::sync::mpsc::Sender<VirtualDesktopEvent>) -> Result<()> {
        self.sender.store(Arc::new(Some(sender)));
        Ok(())
    }
}
