pub mod cli;
pub mod events;
pub mod handlers;
pub mod wallpapers;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::LazyLock;

use seelen_core::state::{DesktopWorkspace, VirtualDesktopMonitor, VirtualDesktops, WorkspaceId};
use seelen_core::system_state::MonitorId;
use slu_utils::{debounce, Debounce};
use windows::Win32::UI::WindowsAndMessaging::{SW_FORCEMINIMIZE, SW_MINIMIZE, SW_RESTORE};

use crate::error::{Result, ResultLogExt};
use crate::event_manager;
use crate::hook::HookManager;
use crate::modules::apps::application::{UserAppWinEvent, UserAppsManager};
use crate::modules::monitors::{MonitorManager, MonitorManagerEvent};
use crate::utils::constants::SEELEN_COMMON;
use crate::utils::lock_free::{SyncHashMap, SyncVec};
use crate::virtual_desktops::wallpapers::WorkspaceWallpapersManager;
use crate::windows_api::window::event::WinEvent;
use crate::windows_api::window::Window;

use events::VirtualDesktopEvent;

static WORKSPACES_MANAGER: LazyLock<SluWorkspacesManager2> =
    LazyLock::new(SluWorkspacesManager2::create);

/// Not a membership list — a window can belong to a workspace (be in its
/// `DesktopWorkspace.windows`) without being here. Tracks *why* a window is
/// currently minimized: only windows minimized by `hide()` because their
/// workspace got hidden are added here. Windows already minimized for another
/// reason (manually by the user, or by the WM as an inactive stack member) are
/// left out, so `restore()` knows not to touch them when their workspace comes
/// back — it only auto-`SW_RESTORE`s windows found in this set.
pub static MINIMIZED_BY_WORKSPACES: LazyLock<scc::HashSet<isize>> =
    LazyLock::new(scc::HashSet::new);

/// Addresses of windows that `restore()` is about to (or just did) `SW_RESTORE`
/// itself. `on_win_event`'s `SystemMinimizeEnd` handler consumes an entry here
/// to recognize "this unminimize was caused by our own workspace restore" and
/// ignore it, instead of misreading it as the user unminimizing the window
/// (which would otherwise re-trigger another `switch_to_id` in a loop).
pub static RESTORED_EVENT_QUEUE: LazyLock<SyncVec<isize>> = LazyLock::new(SyncVec::new);

pub struct SluWorkspacesManager2 {
    pub monitors: SyncHashMap<MonitorId, VirtualDesktopMonitor>,
    pub workspace_index: SyncHashMap<WorkspaceId, MonitorId>,
    pub pinned: SyncVec<isize>,
    /// Count of in-flight `switch_to_id` calls. Used instead of a bool so that
    /// concurrent switches (e.g. on different monitors) don't race: the last
    /// one to finish is the only one allowed to clear the "switching" state.
    switching: AtomicU32,
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
        static SAVE_DEBOUNCER: LazyLock<Debounce<()>> = LazyLock::new(|| {
            debounce(
                |_| {
                    let fun = || {
                        let state: VirtualDesktops = SluWorkspacesManager2::instance().into();
                        let path = SEELEN_COMMON.app_cache_dir().join("workspaces2.json");
                        let mut file = std::fs::File::create(path)?;
                        file.write_all(&serde_json::to_vec(&state)?)?;
                        file.flush()?;
                        log::trace!("desktop workspaces successfully saved");
                        Result::Ok(())
                    };
                    fun().log_error();
                },
                std::time::Duration::from_secs(2),
            )
        });

        SAVE_DEBOUNCER.call(());
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

        // Initialize wallpaper manager and set initial wallpapers
        wallpapers::WorkspaceWallpapersManager::init(&manager);
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
                    // allow resume workspaces correctly on change
                    for addr in &workspace.windows {
                        let _ = MINIMIZED_BY_WORKSPACES.insert(*addr);
                    }
                    workspace.hide(true);
                }
            }
        });

        // create monitors
        for id in MonitorManager::instance().get_cached_ids() {
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
                Self::instance().monitors.get_or_insert(
                    monitor_id,
                    VirtualDesktopMonitor::create,
                    |_| {},
                );
            }
            MonitorManagerEvent::ViewRemoved(_monitor_id) => {
                // Todo: move windows to another monitor, this is probably already done by windows events btw.
                // we don't remove the workspaces items to persist monitor workspaces configuration.
                // Self::instance().monitors.remove(&monitor_id);
            }
            _ => {}
        });

        UserAppsManager::subscribe(|event| match event {
            UserAppWinEvent::Added(addr) => {
                Self::instance().add_to_current_workspace(&Window::from(addr));
            }
            UserAppWinEvent::Removed(addr) => {
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

                // Genuine user action (taskbar click, alt-tab, etc.), not an echo of our own
                // restore(). Let other modules (e.g. the TWM) react to it without having to
                // listen to the raw, indiscriminate SystemMinimizeEnd hook event themselves.
                Self::send(VirtualDesktopEvent::WindowUnminimizedByUser { window: window_id });

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
            WinEvent::SynDebouncedRectChange => {
                let manager = Self::instance();
                if manager.is_pinned(&window_id) {
                    return Ok(());
                }

                let Ok(current_monitor_id) = window.monitor().stable_id() else {
                    return Ok(());
                };

                // Find the monitor whose workspace bookkeeping currently owns this window.
                let recorded_monitor_id = manager.monitors.with_lock(|monitors| {
                    monitors.iter().find_map(|(monitor_id, monitor)| {
                        if monitor
                            .workspaces
                            .iter()
                            .any(|w| w.windows.contains(&window_id))
                        {
                            Some(monitor_id.clone())
                        } else {
                            None
                        }
                    })
                });

                let Some(recorded_monitor_id) = recorded_monitor_id else {
                    // window is not tracked by any workspace, nothing to reconcile
                    return Ok(());
                };

                // Window's physical monitor still matches its workspace's monitor, as it
                // should be, so this is just a regular rect change, not a monitor move.
                if recorded_monitor_id == current_monitor_id {
                    return Ok(());
                }

                let Some(target_workspace_id) = manager
                    .monitors
                    .get(&current_monitor_id, |m| m.active_workspace_id().clone())
                else {
                    return Ok(());
                };

                // Skip if the window is already recorded under the monitor's active
                // workspace (e.g. it was just moved there programmatically via `send_to`).
                // Otherwise this would unconditionally remove+re-add the window, emitting
                // redundant WindowRemoved/WindowAdded events for no actual change.
                let already_there = manager
                    .monitors
                    .get(&current_monitor_id, |m| {
                        m.active_workspace().windows.contains(&window_id)
                    })
                    .unwrap_or(false);

                if !already_there {
                    manager.send_to(&window, &target_workspace_id)?;
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
        let Ok(monitor_id) = window.monitor().stable_id() else {
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
        let switched = self.monitors.with_lock(|monitors| -> Result<bool> {
            {
                let monitor = monitors.get(monitor_id).ok_or("Monitor not found")?;
                if monitor.active_workspace_id() == workspace_id {
                    log::trace!("Already on workspace {workspace_id} on monitor {monitor_id}");
                    return Ok(false);
                }
            }

            self.switching.fetch_add(1, Ordering::SeqCst);
            Self::send(VirtualDesktopEvent::SwitchingDesktop(VirtualDesktops {
                monitors: monitors.clone(),
                pinned: self.pinned.to_vec(),
                switching: true,
            }));

            let monitor = monitors.get_mut(monitor_id).ok_or("Monitor not found")?;
            monitor.active_workspace().hide(false);
            monitor.set_active_workspace(workspace_id)?;
            monitor.active_workspace().restore();

            log::trace!("Switched to workspace {workspace_id} on monitor {monitor_id}");
            Self::send(VirtualDesktopEvent::DesktopChanged {
                monitor: monitor_id.clone(),
                workspace: workspace_id.clone(),
            });

            self.request_save();
            Ok(true)
        })?;

        if switched {
            std::thread::sleep(std::time::Duration::from_millis(300));
            // Only the switch that brings the counter back to 0 is done switching;
            // if it's still > 0, another concurrent switch is still in flight and
            // the "switching" state must remain true.
            if self.switching.fetch_sub(1, Ordering::SeqCst) == 1 {
                Self::send(VirtualDesktopEvent::SwitchingFinished);
            }
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
        // Only move windows that are already tracked; non-interactable windows don't belong
        // to any workspace and should not be added to one by being "moved".
        if !self.contains(window) {
            return Ok(());
        }

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
        self.workspace_index
            .upsert(workspace_id.clone(), monitor_id.clone());

        // Set wallpaper to the new workspace
        WorkspaceWallpapersManager::update_workspace_wallpapers_internal(self);

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

    /// Rename a workspace on a specific monitor
    pub fn rename_desktop(
        &self,
        monitor_id: &MonitorId,
        workspace_id: &WorkspaceId,
        name: Option<String>,
    ) -> Result<()> {
        self.monitors
            .get(monitor_id, |monitor| {
                monitor.rename_workspace(workspace_id, name)
            })
            .ok_or("Monitor not found")??;
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
            switching: AtomicU32::new(0),
        }
    }
}

impl From<&SluWorkspacesManager2> for VirtualDesktops {
    fn from(value: &SluWorkspacesManager2) -> Self {
        Self {
            monitors: value.monitors.to_hash_map(),
            pinned: value.pinned.to_vec(),
            switching: value.switching.load(Ordering::SeqCst) > 0,
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
                window.show_window(mode).log_error();
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
                // Push before show_window to avoid a race where SystemMinimizeEnd
                // fires on the hook thread before this thread reaches the push.
                RESTORED_EVENT_QUEUE.push(*addr);
                // use normal show instead async cuz it will keep the order of restoring
                window.show_window(SW_RESTORE).log_error();
            }
            MINIMIZED_BY_WORKSPACES.remove(addr);

            // ensure correct focus
            if idx == len - 1 {
                window.focus().log_error();
            }
        }
    }
}
