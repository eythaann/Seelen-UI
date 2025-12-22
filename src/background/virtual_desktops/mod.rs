pub mod cli;
pub mod events;
pub mod handlers;

use std::collections::HashMap;
use std::fs::File;
use std::sync::LazyLock;

use seelen_core::state::{DesktopWorkspace, VirtualDesktopMonitor, VirtualDesktops, WorkspaceId};
use seelen_core::system_state::MonitorId;
use tokio::io::AsyncWriteExt;
use windows::Win32::UI::WindowsAndMessaging::{SW_FORCEMINIMIZE, SW_MINIMIZE, SW_RESTORE};

use crate::error::{Result, ResultLogExt};
use crate::hook::HookManager;
use crate::modules::apps::application::{UserAppsEvent, UserAppsManager};
use crate::modules::monitors::{MonitorManager, MonitorManagerEvent};
use crate::utils::constants::SEELEN_COMMON;
use crate::utils::lock_free::{SyncHashMap, SyncVec};
use crate::utils::Debouncer;
use crate::windows_api::window::event::WinEvent;
use crate::windows_api::window::Window;
use crate::{event_manager, log_error};

use events::VirtualDesktopEvent;

static WORKSPACES_MANAGER: LazyLock<SluWorkspacesManager2> =
    LazyLock::new(SluWorkspacesManager2::create);

pub static MINIMIZED_BY_WORKSPACES: LazyLock<scc::HashSet<isize>> =
    LazyLock::new(scc::HashSet::new);
pub static RESTORED_EVENT_QUEUE: LazyLock<SyncVec<isize>> = LazyLock::new(SyncVec::new);

pub struct SluWorkspacesManager2 {
    pub monitors: SyncHashMap<MonitorId, VirtualDesktopMonitor>,
    pub workspace_index: SyncHashMap<WorkspaceId, MonitorId>,
    pub pinned: SyncVec<isize>,
}

event_manager!(SluWorkspacesManager2, VirtualDesktopEvent);

impl SluWorkspacesManager2 {
    pub fn instance() -> &'static Self {
        &WORKSPACES_MANAGER
    }

    fn load_stored() -> Result<VirtualDesktops> {
        let path = SEELEN_COMMON.app_cache_dir().join("workspaces2.json");
        let file = File::open(path)?;
        file.lock()?;
        Ok(serde_json::from_reader(file)?)
    }

    fn request_save(&self) {
        static SAVE_DEBOUNCER: LazyLock<Debouncer> =
            LazyLock::new(|| Debouncer::new(std::time::Duration::from_secs(2)));

        SAVE_DEBOUNCER.call(async move || {
            let state: VirtualDesktops = Self::instance().into();
            let path = SEELEN_COMMON.app_cache_dir().join("workspaces2.json");
            let mut file = tokio::fs::File::create(path).await?;
            file.write_all(&serde_json::to_vec(&state)?).await?;
            file.flush().await?;
            log::trace!("desktop workspaces successfully saved");
            Result::Ok(())
        });
    }

    fn create() -> Self {
        let mut manager = Self::from(match Self::load_stored() {
            Ok(mut state) => {
                state.sanitize();
                state
            }
            Err(_) => Default::default(),
        });
        manager.initialize().log_error();
        manager
    }

    /// TODO: try to move windows on others native virtual desktops to only one,
    /// or add a warning message to users.
    fn initialize(&mut self) -> Result<()> {
        // ensure saved windows are still valid.
        // todo: check if thery are on correct monitor
        self.for_each_workspace(|workspace| {
            workspace
                .windows
                .retain(|w| Window::from(*w).is_interactable_and_not_hidden());
        });

        // restore workspaces state
        self.monitors.for_each(|(_, monitor)| {
            for workspace in &monitor.workspaces {
                if &workspace.id == monitor.active_workspace_id() {
                    workspace.restore();
                } else {
                    workspace.hide(true);
                }
            }
        });

        // create monitors
        for view in MonitorManager::instance().read_all_views()? {
            let id = view.primary_target()?.stable_id()?;
            if self.monitors.contains_key(&id) {
                continue;
            }
            self.monitors.upsert(id, VirtualDesktopMonitor::create());
        }

        // scan no added windows, but only add the non minimized ones to the current active workspace
        UserAppsManager::instance()
            .interactable_windows
            .for_each(|data| {
                let window = Window::from(data.hwnd);
                if !self.contains(&window) && !window.is_minimized() {
                    self.add_to_current_workspace(&window);
                }
            });

        MonitorManager::subscribe(|e| match e {
            MonitorManagerEvent::ViewAdded(monitor_id) => {
                Self::instance()
                    .monitors
                    .upsert(monitor_id, VirtualDesktopMonitor::create());
            }
            MonitorManagerEvent::ViewRemoved(monitor_id) => {
                // Todo: move windows to another monitor
                Self::instance().monitors.remove(&monitor_id);
            }
            _ => {}
        });

        UserAppsManager::subscribe(|event| match event {
            UserAppsEvent::WinAdded(addr) => {
                Self::instance().add_to_current_workspace(&Window::from(addr));
            }
            UserAppsEvent::WinRemoved(addr) => {
                Self::instance().remove(&Window::from(addr));
            }
            _ => {}
        });

        let eid = HookManager::subscribe(|(event, origin)| {
            Self::on_win_event(event, origin).log_error();
        });
        HookManager::set_event_handler_priority(&eid, 2);
        Ok(())
    }

    fn on_win_event(event: WinEvent, window: Window) -> Result<()> {
        let window_id = window.address();
        match event {
            WinEvent::SystemMinimizeEnd => {
                // Check if the window was restored by our workspace system
                let mut found = false;
                RESTORED_EVENT_QUEUE.retain(|w| {
                    if !found && w == &window_id {
                        found = true;
                        return false;
                    }
                    true
                });

                // If found in the queue, it was restored by us, so ignore the event
                if found {
                    return Ok(());
                }

                let manager = Self::instance();
                if let Ok(workspace_id) = window.workspace_id() {
                    let monitor_id = manager.get_monitor_of_workspace(&workspace_id);
                    // Restore workspace if the window was unminimized by the user via alt+tab or others
                    manager.switch_to_id(&monitor_id, &workspace_id)?;
                } else if !manager.is_pinned(&window_id) && window.is_interactable_and_not_hidden()
                {
                    // Add minimized windows during the scanning, to the current active workspace
                    manager.add_to_current_workspace(&window);
                }
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                let manager = Self::instance();
                let mut updated = false;

                // Update z-order: move focused window to end of the list
                manager.monitors.for_each(|(_, monitor)| {
                    for workspace in &mut monitor.workspaces {
                        if workspace.windows.contains(&window_id) {
                            workspace.windows.retain(|w| w != &window_id);
                            workspace.windows.push(window_id);
                            updated = true;
                        }
                    }
                });

                if updated {
                    manager.request_save();
                }
            }
            WinEvent::SyntheticMonitorChanged => {
                let manager = Self::instance();
                if manager.contains(&window) && !manager.is_pinned(&window_id) {
                    manager.remove(&window);
                    manager.add_to_current_workspace(&window);
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn for_each_workspace<F: Fn(&mut DesktopWorkspace)>(&mut self, f: F) {
        self.monitors.for_each(|(_, monitor)| {
            for workspace in &mut monitor.workspaces {
                f(workspace);
            }
        });
    }

    pub fn get_monitor_of_workspace(&self, workspace_id: &WorkspaceId) -> MonitorId {
        self.workspace_index
            .get(workspace_id, |x| x.clone())
            .expect("workspace_index is broken")
    }

    pub fn is_pinned(&self, window_id: &isize) -> bool {
        self.pinned.contains(window_id)
    }

    fn contains(&self, window: &Window) -> bool {
        let window_id = window.address();
        self.is_pinned(&window_id) || {
            self.monitors.any(|(_, monitor)| {
                monitor
                    .workspaces
                    .iter()
                    .any(|w| w.windows.contains(&window_id))
            })
        }
    }

    fn add_to_current_workspace(&self, window: &Window) {
        let window_id = window.address();

        // Get monitor ID with fallback to pinned list
        let Ok(monitor_id) = window.monitor().stable_id2() else {
            // As fallback we gonna add the window to the pinned list.
            // If getting monitor id continues to fail, this won't be able to be unpinned.
            if !self.pinned.contains(&window_id) {
                log::trace!("adding {window} to pinned list");
                self.pinned.push(window_id);
            }
            return;
        };

        // Get or create monitor and add window to active workspace
        let result = self.monitors.get_or_insert(
            monitor_id.clone(),
            VirtualDesktopMonitor::create,
            |monitor| {
                let active_workspace = monitor.active_workspace_mut();
                if active_workspace.windows.contains(&window_id) {
                    return None;
                }

                log::trace!("adding {window} to workspace {}", active_workspace.id);
                active_workspace.windows.push(window_id);
                Some(active_workspace.id.clone())
            },
        );

        // Update workspace index and send event outside of the monitor lock
        if let Some(workspace_id) = result {
            self.workspace_index
                .upsert(workspace_id.clone(), monitor_id);

            Self::send(VirtualDesktopEvent::WindowAdded {
                window: window_id,
                desktop: workspace_id,
            });
            self.request_save();
        }
    }

    fn remove(&self, window: &Window) {
        let window_id = window.address();
        log::trace!("Removing {window} from workspaces");

        // Remove from pinned list
        self.pinned.retain(|w| w != &window_id);

        // Remove from all workspaces
        self.monitors.for_each(|(_, monitor)| {
            for workspace in &mut monitor.workspaces {
                workspace.windows.retain(|w| w != &window_id);
            }
        });

        Self::send(VirtualDesktopEvent::WindowRemoved { window: window_id });
        self.request_save();
    }

    /// Switch to a workspace by ID on a specific monitor
    pub fn switch_to_id(&self, monitor_id: &MonitorId, workspace_id: &WorkspaceId) -> Result<()> {
        let changed = self
            .monitors
            .get(monitor_id, |monitor| {
                if monitor.active_workspace_id() == workspace_id {
                    return Ok(false);
                }

                monitor.active_workspace().hide(false);
                monitor.set_active_workspace(workspace_id)?;
                monitor.active_workspace().restore();
                Result::Ok(true)
            })
            .ok_or("Monitor not found")??;

        if changed {
            log::trace!("Switched to workspace {workspace_id} on monitor {monitor_id}");
            Self::send(VirtualDesktopEvent::DesktopChanged {
                monitor: monitor_id.clone(),
                workspace: workspace_id.clone(),
            });
            self.request_save();
        }

        Ok(())
    }

    /// Switch to a workspace by index on a specific monitor
    pub fn switch_to(&self, monitor_id: &MonitorId, index: usize) -> Result<()> {
        let workspace_id = self
            .monitors
            .get(monitor_id, |monitor| {
                monitor
                    .workspaces
                    .get(index)
                    .map(|w| w.id.clone())
                    .ok_or_else(|| format!("Workspace index {} not found", index))
            })
            .ok_or("Monitor not found")??;
        self.switch_to_id(monitor_id, &workspace_id)
    }

    /// Send a window to a specific workspace
    pub fn send_to(&self, window: &Window, workspace_id: &WorkspaceId) -> Result<()> {
        let monitor_id = self.get_monitor_of_workspace(workspace_id);
        let window_id = window.address();

        // Remove window from current workspace
        self.monitors.for_each(|(_, monitor)| {
            for workspace in &mut monitor.workspaces {
                workspace.windows.retain(|w| w != &window_id);
            }
        });

        // Add window to target workspace
        self.monitors
            .get(&monitor_id, |monitor| {
                let target_workspace = monitor
                    .workspaces
                    .iter_mut()
                    .find(|w| &w.id == workspace_id)
                    .ok_or("Workspace not found in monitor")?;

                target_workspace.windows.push(window_id);

                // Hide window if target workspace is not active
                if monitor.active_workspace_id() != workspace_id {
                    window.show_window(SW_MINIMIZE).ok();
                    let _ = MINIMIZED_BY_WORKSPACES.insert(window_id);
                }

                Self::send(VirtualDesktopEvent::WindowMoved {
                    window: window_id,
                    desktop: workspace_id.clone(),
                });
                self.request_save();
                Ok(())
            })
            .ok_or("Monitor not found")?
    }

    /// Create a new workspace on a specific monitor
    pub fn create_desktop(&self, monitor_id: &MonitorId) -> Result<WorkspaceId> {
        let workspace_id = self
            .monitors
            .get(monitor_id, |monitor| monitor.add_workspace())
            .ok_or("Monitor not found")?;

        // Update workspace index
        self.workspace_index
            .upsert(workspace_id.clone(), monitor_id.clone());
        Self::send(VirtualDesktopEvent::DesktopCreated(workspace_id.clone()));
        self.request_save();
        Ok(workspace_id)
    }

    /// Destroy a workspace on a specific monitor
    pub fn destroy_desktop(
        &self,
        monitor_id: &MonitorId,
        workspace_id: &WorkspaceId,
    ) -> Result<()> {
        self.monitors
            .get(monitor_id, |monitor| {
                let was_active = monitor.active_workspace_id() == workspace_id;
                // Remove the workspace (this moves windows to the previous workspace)
                monitor.remove_workspace(workspace_id)?;
                // If the removed workspace was active, restore the new active workspace
                if was_active {
                    monitor.active_workspace().restore();
                }
                Result::Ok(())
            })
            .ok_or("Monitor not found")??;

        // Remove from workspace index
        self.workspace_index.remove(workspace_id);
        Self::send(VirtualDesktopEvent::DesktopDestroyed(workspace_id.clone()));
        self.request_save();
        Ok(())
    }
}

impl From<VirtualDesktops> for SluWorkspacesManager2 {
    fn from(value: VirtualDesktops) -> Self {
        let mut workspace_index = HashMap::new();
        for (mid, m) in &value.monitors {
            for w in &m.workspaces {
                workspace_index.insert(w.id.clone(), mid.clone());
            }
        }

        Self {
            monitors: SyncHashMap::from(value.monitors),
            workspace_index: SyncHashMap::from(workspace_index),
            pinned: SyncVec::from(value.pinned),
        }
    }
}

impl From<&SluWorkspacesManager2> for VirtualDesktops {
    fn from(value: &SluWorkspacesManager2) -> Self {
        Self {
            monitors: value.monitors.to_hash_map(),
            pinned: value.pinned.to_vec(),
        }
    }
}

pub trait DesktopWorkspaceExt {
    fn hide(&self, force: bool);
    fn restore(&self);
}

impl DesktopWorkspaceExt for DesktopWorkspace {
    fn hide(&self, force: bool) {
        let mode = if force { SW_FORCEMINIMIZE } else { SW_MINIMIZE };
        for addr in &self.windows {
            let window = Window::from(*addr);
            if window.is_window() && !window.is_minimized() {
                let _ = MINIMIZED_BY_WORKSPACES.insert(window.address());
                log_error!(window.show_window(mode));
            }
        }
    }

    fn restore(&self) {
        let len = self.windows.len();
        for (idx, addr) in self.windows.iter().enumerate() {
            let window = Window::from(*addr);
            let is_minimized = window.is_minimized();

            // avoid restore windows manually minimized by the user
            if is_minimized && !MINIMIZED_BY_WORKSPACES.contains(addr) {
                continue;
            }

            if is_minimized {
                // use normal show instead async cuz it will keep the order of restoring
                log_error!(window.show_window(SW_RESTORE));
                RESTORED_EVENT_QUEUE.push(*addr);
            }
            MINIMIZED_BY_WORKSPACES.remove(addr);

            // ensure correct focus
            if idx == len - 1 {
                log_error!(window.focus());
            }
        }
    }
}
