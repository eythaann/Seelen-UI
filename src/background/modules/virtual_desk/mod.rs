mod cli;
mod native;
mod workspaces;

use arc_swap::ArcSwap;
use lazy_static::lazy_static;
use seelen_core::state::VirtualDesktopStrategy;
use serde::Serialize;
use std::sync::Arc;

use crate::{error_handler::Result, state::application::FULL_STATE};

lazy_static! {
    pub static ref VIRTUAL_DESKTOP_MANAGER: Arc<ArcSwap<VirtualDesktopManager>> =
        Arc::new(ArcSwap::from_pointee(
            match FULL_STATE.load().settings().virtual_desktop_strategy {
                VirtualDesktopStrategy::Native =>
                    VirtualDesktopManager::Native(native::NativeVirtualDesktopManager::new()),
                VirtualDesktopStrategy::Seelen =>
                    VirtualDesktopManager::Seelen(workspaces::SeelenWorkspacesManager::new()),
            }
        ));
}

trait VirtualDesktopTrait: std::fmt::Debug + Clone {
    fn id(&self) -> String;
    fn name(&self) -> Option<String>;
}

trait VirtualDesktopManagerTrait {
    fn create_desktop(&self) -> Result<()>;

    fn get(&self, idx: usize) -> Result<Option<VirtualDesktop>>;
    fn get_all(&self) -> Result<Vec<VirtualDesktop>>;
    /// All windows should be in one virtual desktop, that's why this doesn't return `Option`
    fn get_by_window(&self, window: isize) -> Result<VirtualDesktop>;

    fn get_current(&self) -> Result<VirtualDesktop>;
    fn get_current_idx(&self) -> Result<usize>;

    fn switch_to(&self, idx: usize) -> Result<()>;
    fn send_to(&self, idx: usize, window: isize) -> Result<()>;

    fn pin_window(&self, window: isize) -> Result<()>;
    fn unpin_window(&self, window: isize) -> Result<()>;
    fn is_pinned_window(&self, window: isize) -> Result<bool>;

    fn listen_events(&self, cb: std::sync::mpsc::Sender<VirtualDesktopEvent>) -> Result<()>;

    fn uses_cloak(&self) -> bool;
}

#[derive(Debug, Clone)]
pub enum VirtualDesktop {
    Native(native::NativeVirtualDesktop),
    Seelen(workspaces::SeelenWorkspace),
}

#[derive(Serialize)]
pub struct SerializableVirtualDesktop {
    id: String,
    name: Option<String>,
}

impl VirtualDesktop {
    pub fn id(&self) -> String {
        match self {
            VirtualDesktop::Native(d) => d.id(),
            VirtualDesktop::Seelen(d) => d.id(),
        }
    }

    pub fn name(&self) -> Option<String> {
        match self {
            VirtualDesktop::Native(d) => d.name(),
            VirtualDesktop::Seelen(d) => d.name(),
        }
    }

    pub fn as_serializable(&self) -> SerializableVirtualDesktop {
        SerializableVirtualDesktop {
            id: self.id(),
            name: self.name().clone(),
        }
    }
}

#[derive(Debug)]
pub enum VirtualDesktopManager {
    Native(native::NativeVirtualDesktopManager),
    Seelen(workspaces::SeelenWorkspacesManager),
}

impl VirtualDesktopManager {
    pub fn create_desktop(&self) -> Result<()> {
        match self {
            VirtualDesktopManager::Native(m) => m.create_desktop(),
            VirtualDesktopManager::Seelen(m) => m.create_desktop(),
        }
    }

    pub fn get(&self, idx: usize) -> Result<Option<VirtualDesktop>> {
        match self {
            VirtualDesktopManager::Native(m) => m.get(idx),
            VirtualDesktopManager::Seelen(m) => m.get(idx),
        }
    }

    pub fn get_all(&self) -> Result<Vec<VirtualDesktop>> {
        match self {
            VirtualDesktopManager::Native(m) => m.get_all(),
            VirtualDesktopManager::Seelen(m) => m.get_all(),
        }
    }

    pub fn get_by_window(&self, window: isize) -> Result<VirtualDesktop> {
        match self {
            VirtualDesktopManager::Native(m) => m.get_by_window(window),
            VirtualDesktopManager::Seelen(m) => m.get_by_window(window),
        }
    }

    pub fn get_current(&self) -> Result<VirtualDesktop> {
        match self {
            VirtualDesktopManager::Native(m) => m.get_current(),
            VirtualDesktopManager::Seelen(m) => m.get_current(),
        }
    }

    pub fn get_current_idx(&self) -> Result<usize> {
        match self {
            VirtualDesktopManager::Native(m) => m.get_current_idx(),
            VirtualDesktopManager::Seelen(m) => m.get_current_idx(),
        }
    }

    pub fn switch_to(&self, idx: usize) -> Result<()> {
        match self {
            VirtualDesktopManager::Native(m) => m.switch_to(idx),
            VirtualDesktopManager::Seelen(m) => m.switch_to(idx),
        }
    }

    pub fn send_to(&self, idx: usize, window: isize) -> Result<()> {
        match self {
            VirtualDesktopManager::Native(m) => m.send_to(idx, window),
            VirtualDesktopManager::Seelen(m) => m.send_to(idx, window),
        }
    }

    pub fn pin_window(&self, window: isize) -> Result<()> {
        match self {
            VirtualDesktopManager::Native(m) => m.pin_window(window),
            VirtualDesktopManager::Seelen(m) => m.pin_window(window),
        }
    }

    pub fn unpin_window(&self, window: isize) -> Result<()> {
        match self {
            VirtualDesktopManager::Native(m) => m.unpin_window(window),
            VirtualDesktopManager::Seelen(m) => m.unpin_window(window),
        }
    }

    pub fn is_pinned_window(&self, window: isize) -> Result<bool> {
        match self {
            VirtualDesktopManager::Native(m) => m.is_pinned_window(window),
            VirtualDesktopManager::Seelen(m) => m.is_pinned_window(window),
        }
    }

    pub fn listen_events(&self, cb: std::sync::mpsc::Sender<VirtualDesktopEvent>) -> Result<()> {
        match self {
            VirtualDesktopManager::Native(m) => m.listen_events(cb),
            VirtualDesktopManager::Seelen(m) => m.listen_events(cb),
        }
    }

    pub fn uses_cloak(&self) -> bool {
        match self {
            VirtualDesktopManager::Native(m) => m.uses_cloak(),
            VirtualDesktopManager::Seelen(m) => m.uses_cloak(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum VirtualDesktopEvent {
    DesktopCreated(VirtualDesktop),
    DesktopDestroyed {
        destroyed: VirtualDesktop,
        fallback: VirtualDesktop,
    },
    DesktopChanged {
        new: VirtualDesktop,
        old: VirtualDesktop,
    },
    DesktopNameChanged(VirtualDesktop, String),
    DesktopWallpaperChanged(VirtualDesktop, String),
    DesktopMoved {
        desktop: VirtualDesktop,
        old_index: usize,
        new_index: usize,
    },
    /// Emitted when a window is moved of the virtual desktop.
    /// If using native implementation, it also will be triggered when the window is created/destroyed
    WindowChanged(isize),
}

pub fn get_vd_manager() -> Arc<VirtualDesktopManager> {
    VIRTUAL_DESKTOP_MANAGER.load_full()
}
