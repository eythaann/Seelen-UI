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
    hook::HOOK_MANAGER,
    log_error,
    seelen_weg::SeelenWeg,
    trace_lock,
    windows_api::{WindowEnumerator, WindowsApi},
    winevent::WinEvent,
};

use super::{VirtualDesktop, VirtualDesktopEvent, VirtualDesktopManagerTrait, VirtualDesktopTrait};
use windows::Win32::Foundation::HWND;
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

    fn remove_window(&mut self, window: isize) -> Result<()> {
        self.windows.retain(|w| *w != window);
        WindowsApi::set_minimize_animation(false)?;
        trace_lock!(HOOK_MANAGER).skip(WinEvent::SystemMinimizeStart, window);
        WindowsApi::minimize_window(HWND(window))?;
        WindowsApi::set_minimize_animation(true)?;
        Ok(())
    }

    fn hide(&mut self) -> Result<()> {
        WindowsApi::set_minimize_animation(false)?;
        let mut hook_manager = trace_lock!(HOOK_MANAGER);
        for window in &self.windows {
            hook_manager.skip(WinEvent::SystemMinimizeStart, *window);
            WindowsApi::minimize_window(HWND(*window))?;
        }
        WindowsApi::set_minimize_animation(true)?;
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        WindowsApi::set_minimize_animation(false)?;
        let mut hook_manager = trace_lock!(HOOK_MANAGER);
        for window in &self.windows {
            let hwnd = HWND(*window);
            // if is switching by restored window on other workspace it will be already shown
            if WindowsApi::is_iconic(hwnd) {
                hook_manager.skip(WinEvent::SystemMinimizeEnd, hwnd.0);
                WindowsApi::restore_window(hwnd)?;
            }
        }
        WindowsApi::set_minimize_animation(true)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SeelenWorkspacesManager {
    current: AtomicUsize,
    sender: ArcSwap<Option<std::sync::mpsc::Sender<VirtualDesktopEvent>>>,
    workspaces: Mutex<Vec<SeelenWorkspace>>,
    pinned: Mutex<Vec<isize>>,
}

fn none_err() -> AppError {
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
        log_error!(manager.load());
        manager
    }

    /// TODO: try to move windows on others native virtual desktops to only one.
    fn load(&self) -> Result<()> {
        let mut workspaces = self.workspaces();
        let workspace = workspaces.get_mut(self._current()).ok_or_else(none_err)?;
        for hwnd in WindowEnumerator::new_refreshed()? {
            if SeelenWeg::should_be_added(hwnd) && !WindowsApi::is_iconic(hwnd) {
                workspace.windows.push(hwnd.0);
            }
        }
        Ok(())
    }

    /// should be called on a thread to avoid deadlocks
    pub fn on_win_event(&self, event: WinEvent, origin: HWND) -> Result<()> {
        match event {
            WinEvent::SystemMinimizeStart | WinEvent::ObjectDestroy | WinEvent::ObjectHide => {
                let mut workspaces = self.workspaces();
                let workspace = workspaces.get_mut(self._current()).ok_or_else(none_err)?;
                if workspace.windows.contains(&origin.0) {
                    log::trace!("removing window: {}", origin.0);
                    workspace.windows.retain(|w| *w != origin.0);
                }
            }
            WinEvent::SystemMinimizeEnd => {
                let owner_idx = {
                    let workspaces = self.workspaces();
                    workspaces
                        .iter()
                        .position(|w| w.windows.contains(&origin.0))
                };
                if let Some(owner_idx) = owner_idx {
                    self.switch_to(owner_idx)?;
                } else if SeelenWeg::should_be_added(origin) {
                    log::trace!("adding window to workspace: {}", origin.0);
                    let mut workspaces = self.workspaces();
                    let workspace = workspaces.get_mut(self._current()).ok_or_else(none_err)?;
                    workspace.windows.push(origin.0);
                }
            }
            WinEvent::ObjectCreate | WinEvent::ObjectShow => {
                if SeelenWeg::should_be_added(origin) {
                    log::trace!("adding window to workspace: {}", origin.0);
                    let mut workspaces = self.workspaces();
                    let workspace = workspaces.get_mut(self._current()).ok_or_else(none_err)?;
                    workspace.windows.push(origin.0);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn _current(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }

    fn workspaces(&self) -> MutexGuard<'_, Vec<SeelenWorkspace>> {
        trace_lock!(self.workspaces)
    }

    fn pinned(&self) -> MutexGuard<'_, Vec<isize>> {
        trace_lock!(self.pinned)
    }

    fn emit(&self, event: VirtualDesktopEvent) -> Result<()> {
        if let Some(sender) = self.sender.load().as_ref() {
            sender.send(event).map_err(|_| "Failed to send event")?;
        }
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
        self.workspaces().push(desk.clone());
        self.emit(VirtualDesktopEvent::DesktopCreated(desk.into()))?;
        Ok(())
    }

    fn get(&self, idx: usize) -> Result<Option<VirtualDesktop>> {
        Ok(self.workspaces().get(idx).map(Into::into))
    }

    fn get_by_window(&self, window: isize) -> Result<VirtualDesktop> {
        if self.is_pinned_window(window)? {
            return self.get_current();
        }
        let vd = {
            self.workspaces()
                .iter()
                .find(|w| w.windows.contains(&window))
                .map(Into::into)
        };
        vd.or_else(|| self.get_current().ok()).ok_or_else(none_err)
    }

    fn get_all(&self) -> Result<Vec<VirtualDesktop>> {
        Ok(self.workspaces().iter().map(Into::into).collect())
    }

    fn get_current(&self) -> Result<VirtualDesktop> {
        Ok(self
            .workspaces()
            .get(self._current())
            .ok_or_else(none_err)?
            .into())
    }

    fn get_current_idx(&self) -> Result<usize> {
        Ok(self._current())
    }

    fn switch_to(&self, idx: usize) -> Result<()> {
        {
            let len = self.workspaces().len();
            if idx >= len {
                // temporal until implement a UI to create seelen workspaces
                self.create_many_desktop((idx + 1) - len)?;
                // return Err("Tried to switch to non-existent workspace".into());
            }
        }

        let mut workspaces = self.workspaces();
        let old = {
            let old = workspaces.get_mut(self._current()).ok_or_else(none_err)?;
            old.hide()?;
            old.clone()
        };

        self.current.store(idx, Ordering::SeqCst);

        let new = workspaces.get(self._current()).ok_or_else(none_err)?;
        new.restore()?;

        self.emit(VirtualDesktopEvent::DesktopChanged {
            new: new.into(),
            old: old.into(),
        })?;
        Ok(())
    }

    fn send_to(&self, idx: usize, window: isize) -> Result<()> {
        if idx >= self.workspaces().len() {
            // temporal until implement a UI to create seelen workspaces
            self.create_many_desktop((idx + 1) - self.workspaces().len())?;
            // return Err("Tried to send to non-existent workspace".into());
        }
        let mut workspaces = self.workspaces();
        {
            let old_idx = workspaces
                .iter()
                .position(|w| w.windows.contains(&window))
                .ok_or_else(none_err)?;
            let old = workspaces.get_mut(old_idx).ok_or_else(none_err)?;
            old.remove_window(window)?;
        }
        {
            let new = workspaces.get_mut(idx).ok_or_else(none_err)?;
            new.windows.push(window);
        }
        self.emit(VirtualDesktopEvent::WindowChanged(window))?;
        Ok(())
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
