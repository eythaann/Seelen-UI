pub mod cli;
pub mod events;
pub mod handlers;
mod win_hook;

use std::fs::File;
use std::sync::{Arc, LazyLock};

use parking_lot::{Mutex, MutexGuard};
use seelen_core::state::{DesktopWorkspace, VirtualDesktopMonitor, VirtualDesktops, WorkspaceId};
use seelen_core::system_state::MonitorId;
use tokio::io::AsyncWriteExt;
use windows::Win32::UI::WindowsAndMessaging::{SW_FORCEMINIMIZE, SW_MINIMIZE, SW_RESTORE};

use crate::error::Result;
use crate::utils::constants::SEELEN_COMMON;
use crate::utils::Debouncer;
use crate::virtual_desktops::events::VirtualDesktopEvent;
use crate::windows_api::window::Window;
use crate::{event_manager, log_error, trace_lock};

static SAVE_DEBOUNCER: LazyLock<Debouncer> =
    LazyLock::new(|| Debouncer::new(std::time::Duration::from_secs(2)));

pub static WORKSPACES_MANAGER: LazyLock<Arc<Mutex<SluWorkspacesManager>>> =
    LazyLock::new(|| Arc::new(Mutex::new(SluWorkspacesManager::init())));

pub static MINIMIZED_BY_USER: LazyLock<scc::HashSet<isize>> = LazyLock::new(scc::HashSet::new);
pub static MINIMIZED_BY_WORKSPACES: LazyLock<scc::HashSet<isize>> =
    LazyLock::new(scc::HashSet::new);

pub struct SluWorkspacesManager(VirtualDesktops);

event_manager!(SluWorkspacesManager, VirtualDesktopEvent);

impl SluWorkspacesManager {
    pub fn load_stored() -> Result<VirtualDesktops> {
        let path = SEELEN_COMMON.app_cache_dir().join("workspaces.json");
        let file = File::open(path)?;
        file.lock()?;
        Ok(serde_json::from_reader(file)?)
    }

    pub fn save(&self) {
        let state = self.0.clone();
        SAVE_DEBOUNCER.call(async move || {
            let path = SEELEN_COMMON.app_cache_dir().join("workspaces.json");
            let mut file = tokio::fs::File::create(path).await?;
            file.write_all(&serde_json::to_vec(&state)?).await?;
            file.flush().await?;
            log::trace!("desktop workspaces successfully saved");
            Result::Ok(())
        });
    }

    fn init() -> Self {
        let mut manager = Self(match Self::load_stored() {
            Ok(mut state) => {
                state.sanitize();
                state
            }
            Err(_) => Default::default(),
        });

        // ensure saved windows are still valid.
        for workspace in manager.iter_workspaces_mut() {
            workspace
                .windows
                .retain(|w| Self::should_be_added(&Window::from(*w)));
        }

        Self::init_hook_listener();
        manager
    }

    pub fn is_pinned(&self, window_id: &isize) -> bool {
        self.0.pinned.contains(window_id)
    }

    fn hide_workspace(workspace: &DesktopWorkspace, force: bool) {
        let mode = if force { SW_FORCEMINIMIZE } else { SW_MINIMIZE };
        for addr in &workspace.windows {
            let window = Window::from(*addr);
            if window.is_window() && !window.is_minimized() {
                let _ = MINIMIZED_BY_WORKSPACES.insert(window.address());
                log_error!(window.show_window_async(mode));
            }
        }
    }

    fn restore_workspace(workspace: &DesktopWorkspace) {
        let len = workspace.windows.len();
        for (idx, addr) in workspace.windows.iter().enumerate() {
            if MINIMIZED_BY_USER.contains(addr) {
                continue;
            }

            let window = Window::from(*addr);
            if window.is_window() && window.is_minimized() {
                MINIMIZED_BY_WORKSPACES.remove(&window.address());
                // use normal show instead async cuz it will keep the order of restoring
                log_error!(window.show_window(SW_RESTORE));
            }

            // ensure correct focus
            if idx == len - 1 {
                log_error!(window.focus());
            }
        }
    }

    pub fn switch_to_id(
        &mut self,
        monitor_id: &MonitorId,
        workspace_id: &WorkspaceId,
    ) -> Result<()> {
        let monitor = self.get_monitor_mut(monitor_id);

        if &monitor.current_workspace == workspace_id {
            return Ok(()); // nothing to do
        }

        let current = monitor
            .workspaces
            .iter()
            .find(|w| w.id == monitor.current_workspace)
            .ok_or("current workspace not found")?;
        let workspace_to_change = monitor
            .workspaces
            .iter()
            .find(|w| &w.id == workspace_id)
            .ok_or("workspace not found")?;

        monitor.current_workspace = workspace_to_change.id.clone();
        Self::hide_workspace(current, false);
        Self::restore_workspace(workspace_to_change);

        Self::event_tx().send(VirtualDesktopEvent::DesktopChanged {
            monitor: monitor_id.clone(),
            workspace: workspace_id.clone(),
        })?;
        self.save();
        Ok(())
    }

    pub fn switch_to(&mut self, monitor_id: &MonitorId, index: usize) -> Result<()> {
        let monitor = self.get_monitor_mut(monitor_id);
        let current = monitor
            .workspaces
            .iter()
            .find(|w| w.id == monitor.current_workspace)
            .ok_or("current workspace not found")?;

        if let Some(workspace_to_change) = monitor.workspaces.get(index) {
            if workspace_to_change.id == current.id {
                return Ok(()); // nothing to do
            }
            monitor.current_workspace = workspace_to_change.id.clone();
            Self::hide_workspace(current, false);
            Self::restore_workspace(workspace_to_change);
        }

        Self::event_tx().send(VirtualDesktopEvent::DesktopChanged {
            monitor: monitor_id.clone(),
            workspace: monitor.current_workspace.clone(),
        })?;
        self.save();
        Ok(())
    }

    /// will fail if target workspace doesn't exist
    pub fn send_to(
        &mut self,
        monitor_id: &MonitorId,
        w_index: usize,
        window_id: &isize,
    ) -> Result<()> {
        let current_id = match self.workspace_containing_window(window_id) {
            Some(w) => w.id.clone(),
            None => return Ok(()), // unmanaged windows can be moved
        };

        let target_id = {
            let target = self
                .get_monitor_mut(monitor_id)
                .workspaces
                .get_mut(w_index)
                .ok_or("Target workspace not found")?;

            if current_id == target.id {
                return Ok(()); // nothing to do
            }

            target.windows.push(*window_id);
            target.id.clone()
        };

        if let Some(old) = self.iter_workspaces_mut().find(|w| w.id == current_id) {
            old.windows.retain(|w| w != window_id);
        }

        let window = Window::from(*window_id);
        if !window.is_minimized() {
            let _ = MINIMIZED_BY_WORKSPACES.insert(window.address());
            log_error!(window.show_window(SW_FORCEMINIMIZE));
        }

        Self::send(VirtualDesktopEvent::WindowMoved {
            window: *window_id,
            desktop: target_id,
        });
        self.save();

        Ok(())
    }

    /// create a new desktop, and return its id
    pub fn create_desktop(&mut self, monitor_id: &MonitorId) -> WorkspaceId {
        let new_desktop = DesktopWorkspace::create();
        let new_desktop_id = new_desktop.id.clone();
        self.get_monitor_mut(monitor_id)
            .workspaces
            .push(new_desktop);
        self.save();
        Self::send(VirtualDesktopEvent::DesktopCreated(new_desktop_id.clone()));
        new_desktop_id
    }

    pub fn destroy_desktop(&mut self, id: &WorkspaceId) {
        for monitor in self.0.monitors.values_mut() {
            let Some(deleting_idx) = monitor.workspaces.iter().position(|w| &w.id == id) else {
                continue;
            };

            // do not destroy last desktop
            if monitor.workspaces.len() < 2 {
                return;
            }

            let deleted = monitor.workspaces.remove(deleting_idx);
            let fallback_idx = if deleting_idx == 0 {
                0
            } else {
                deleting_idx - 1
            };

            let fallback = &mut monitor.workspaces[fallback_idx];
            fallback.windows.append(&mut deleted.windows.clone());

            // switch to the fallback workspace if deleted one was the active
            if monitor.current_workspace == deleted.id {
                monitor.current_workspace = fallback.id.clone();
                Self::restore_workspace(fallback);
            }

            for addr in deleted.windows {
                Self::send(VirtualDesktopEvent::WindowMoved {
                    window: addr,
                    desktop: fallback.id.clone(),
                });
            }

            Self::send(VirtualDesktopEvent::DesktopChanged {
                monitor: monitor.id.clone(),
                workspace: fallback.id.clone(),
            });
            Self::send(VirtualDesktopEvent::DesktopDestroyed(deleted.id));
            break;
        }

        self.save();
    }
}

// getters and setters
impl SluWorkspacesManager {
    pub fn desktops(&self) -> &VirtualDesktops {
        &self.0
    }

    pub fn desktops_mut(&mut self) -> &mut VirtualDesktops {
        &mut self.0
    }

    pub fn iter_workspaces(&self) -> impl Iterator<Item = &DesktopWorkspace> {
        self.desktops()
            .monitors
            .values()
            .flat_map(|monitor| monitor.workspaces.iter())
    }

    pub fn iter_workspaces_mut(&mut self) -> impl Iterator<Item = &mut DesktopWorkspace> {
        self.desktops_mut()
            .monitors
            .values_mut()
            .flat_map(|monitor| monitor.workspaces.iter_mut())
    }

    // will insert new monitor if not found
    pub fn get_monitor_mut(&mut self, id: &MonitorId) -> &mut VirtualDesktopMonitor {
        self.desktops_mut()
            .monitors
            .entry(id.clone())
            .or_insert_with(|| VirtualDesktopMonitor::create(id.clone()))
    }

    pub fn get_active_workspace_id(&mut self, monitor_id: &MonitorId) -> &WorkspaceId {
        let monitor = self.get_monitor_mut(monitor_id);
        &monitor.current_workspace
    }

    pub fn get_active_workspace_mut(&mut self, monitor_id: &MonitorId) -> &mut DesktopWorkspace {
        let monitor = self.get_monitor_mut(monitor_id);
        monitor
            .workspaces
            .iter_mut()
            .find(|w| w.id == monitor.current_workspace)
            .expect("active workspace not found")
    }

    pub fn workspace_containing_window(&self, window_id: &isize) -> Option<&DesktopWorkspace> {
        if self.is_pinned(window_id) {
            return None;
        }
        self.iter_workspaces()
            .find(|w| w.windows.contains(window_id))
    }

    pub fn workspace_containing_window_mut(
        &mut self,
        window_id: &isize,
    ) -> Option<&mut DesktopWorkspace> {
        if self.is_pinned(window_id) {
            return None;
        }
        self.iter_workspaces_mut()
            .find(|w| w.windows.contains(window_id))
    }
}

pub fn get_vd_manager<'a>() -> MutexGuard<'a, SluWorkspacesManager> {
    trace_lock!(WORKSPACES_MANAGER)
}
