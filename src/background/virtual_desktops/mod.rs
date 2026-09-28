pub mod bridge;
pub mod cli;
pub mod events;
pub mod handlers;
pub mod wallpapers;

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU32, Ordering};

use seelen_core::state::{DesktopWorkspace, VirtualDesktopMonitor, VirtualDesktops, WorkspaceId};
use seelen_core::system_state::MonitorId;
use windows::Win32::UI::WindowsAndMessaging::{SW_FORCEMINIMIZE, SW_MINIMIZE, SW_RESTORE};

use crate::error::{Result, ResultLogExt};
use crate::event_manager;
use crate::hook::HookManager;
use crate::modules::apps::application::{UserAppWinEvent, UserAppsManager};
use crate::modules::monitors::{MonitorManager, MonitorManagerEvent};
use crate::utils::lock_free::{SyncHashMap, SyncVec};
use crate::virtual_desktops::wallpapers::WorkspaceWallpapersManager;
use crate::windows_api::window::Window;
use crate::windows_api::window::event::WinEvent;

use events::VirtualDesktopEvent;

static WORKSPACES_MANAGER: LazyLock<VdManager> = LazyLock::new(VdManager::create);

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

/// Lock order: `monitors` -> (`pinned` | `workspace_index` | `window_index`) -> UserApps
/// `interactable_windows`. Never take `monitors` while holding any of the others.
pub struct VdManager {
    pub monitors: SyncHashMap<MonitorId, VirtualDesktopMonitor>,

    /// Only mutated while holding the `monitors` lock, so they never diverge from it.
    pub workspace_index: SyncHashMap<WorkspaceId, MonitorId>,
    pub window_index: SyncHashMap<isize, WorkspaceId>,

    pub pinned: SyncVec<isize>,
    /// Count of in-flight `switch_to_id` calls. Used instead of a bool so that
    /// concurrent switches (e.g. on different monitors) don't race: the last
    /// one to finish is the only one allowed to clear the "switching" state.
    switching: AtomicU32,
}

event_manager!(VdManager, VirtualDesktopEvent);

impl VdManager {
    pub fn instance() -> &'static Self {
        &WORKSPACES_MANAGER
    }

    fn create() -> Self {
        let mut manager = Self::from(match bridge::load_stored() {
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
        // todo: check if they are on correct monitor
        self.for_each_workspace(|workspace| {
            workspace
                .windows
                .retain(|w| Window::from(*w).is_interactable_and_not_hidden());
        });
        self.pinned
            .retain(|w| Window::from(*w).is_interactable_and_not_hidden());
        self.rebuild_indexes();

        // restore workspaces state
        self.monitors.for_each(|(_, monitor)| {
            for row in monitor.workspaces.rows() {
                for workspace in row {
                    if &workspace.id == monitor.active_workspace_id() {
                        workspace.restore();
                    } else {
                        // allow resume workspaces correctly on change
                        for addr in &workspace.windows {
                            let _ = MINIMIZED_BY_WORKSPACES.insert_sync(*addr);
                        }
                        workspace.hide(true);
                    }
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

        // scan no added windows, but only add the non minimized ones to the current active workspace.
        // Collected first to not hold the interactables lock while taking the monitors lock (see lock order).
        let interactables = UserAppsManager::instance()
            .interactable_windows
            .map(|data| data.hwnd);
        for hwnd in interactables {
            let window = Window::from(hwnd);
            if !window.is_minimized() {
                self.add_to_current_workspace(&window);
            }
        }

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

        // Save state on any change
        Self::subscribe(|_e| {
            bridge::request_save();
        });
        Ok(())
    }

    pub fn for_each_workspace<F: Fn(&mut DesktopWorkspace)>(&mut self, f: F) {
        self.monitors.for_each(|(_, monitor)| {
            for row in monitor.workspaces.rows_mut() {
                for workspace in row {
                    f(workspace);
                }
            }
        });
    }

    pub fn get_workspace_of_window(&self, window_id: isize) -> Option<WorkspaceId> {
        self.window_index.get(&window_id, |x| x.clone())
    }

    pub fn get_monitor_of_workspace(&self, workspace_id: &WorkspaceId) -> Option<MonitorId> {
        self.workspace_index.get(workspace_id, |x| x.clone())
    }

    pub fn is_pinned(&self, window_id: &isize) -> bool {
        self.pinned.contains(window_id)
    }

    /// Must be called while holding the monitors lock, so indexes never diverge from it.
    fn replace_indexes(&self, monitors: &HashMap<MonitorId, VirtualDesktopMonitor>) {
        let (workspace_index, window_index) = bridge::get_indexes(monitors);
        self.workspace_index.replace(workspace_index);
        self.window_index.replace(window_index);
    }

    fn rebuild_indexes(&self) {
        self.monitors
            .with_lock(|monitors| self.replace_indexes(monitors));
    }

    fn add_to_current_workspace(&self, window: &Window) {
        let window_id = window.address();
        // Win32 calls are done before taking the lock
        let monitor_id = window.monitor().stable_id();

        // Checks and insertion are done under the monitors lock, so this can't interleave
        // with other add/remove/send_to/switch and track the window twice or as a ghost.
        let added = self.monitors.with_lock(|monitors| {
            // A window belongs to one workspace only, don't track it twice.
            if self.pinned.contains(&window_id) || self.window_index.contains_key(&window_id) {
                return None;
            }

            // The window could have been removed from the interactables (and so from here)
            // after the caller checked it, in that case tracking it would leave a ghost window.
            if !UserAppsManager::instance().contains_win(window) {
                return None;
            }

            let Ok(monitor_id) = monitor_id else {
                // As fallback we gonna add the window to the pinned list.
                // If getting monitor id continues to fail, this won't be able to be unpinned.
                self.pinned.push(window_id);
                return None;
            };

            let workspace_id = {
                let active_workspace = monitors
                    .entry(monitor_id.clone())
                    .or_insert_with(VirtualDesktopMonitor::create)
                    .active_workspace_mut();
                active_workspace.windows.push(window_id);
                active_workspace.id.clone()
            };

            self.workspace_index
                .upsert(workspace_id.clone(), monitor_id);
            self.window_index.upsert(window_id, workspace_id.clone());
            Some(workspace_id)
        });

        if let Some(workspace_id) = added {
            log::trace!("added {window} to workspace {workspace_id}");
            Self::send(VirtualDesktopEvent::WindowAdded {
                window: window_id,
                desktop: workspace_id,
            });
        }
    }

    fn remove(&self, window: &Window) {
        let window_id = window.address();

        let was_tracked = self.monitors.with_lock(|monitors| {
            let mut was_tracked = self.window_index.remove(&window_id).is_some();
            self.pinned.retain(|w| {
                let is_it = w == &window_id;
                was_tracked |= is_it;
                !is_it
            });
            for monitor in monitors.values_mut() {
                for row in monitor.workspaces.rows_mut() {
                    for workspace in row {
                        workspace.windows.retain(|w| w != &window_id);
                    }
                }
            }
            was_tracked
        });

        if was_tracked {
            log::trace!("removed {window} from workspaces");
            Self::send(VirtualDesktopEvent::WindowRemoved { window: window_id });
        }
    }

    /// Switch to a workspace by ID, the owning monitor is resolved from the workspace index
    pub fn switch_to_id(&self, workspace_id: &WorkspaceId) -> Result<()> {
        let monitor_id = self
            .get_monitor_of_workspace(workspace_id)
            .ok_or("Workspace not found")?;

        let switched = self.monitors.with_lock(|monitors| -> Result<bool> {
            {
                let monitor = monitors.get(&monitor_id).ok_or("Monitor not found")?;
                if monitor.active_workspace_id() == workspace_id {
                    log::trace!("Already on workspace {workspace_id} on monitor {monitor_id}");
                    return Ok(false);
                }
            }

            // snapshot of the state before switching, sent along the switching event
            let snapshot = monitors.clone();

            let monitor = monitors.get_mut(&monitor_id).ok_or("Monitor not found")?;
            let previous_id = monitor.active_workspace_id().clone();
            // Set the new active workspace before any side effect, so if it fails
            // (e.g. destroyed meanwhile) nothing got hidden and `switching` is untouched.
            monitor.set_active_workspace(workspace_id)?;

            self.switching.fetch_add(1, Ordering::SeqCst);
            Self::send(VirtualDesktopEvent::SwitchingDesktop(VirtualDesktops {
                monitors: snapshot,
                pinned: self.pinned.to_vec(),
                switching: true,
            }));

            if let Some(previous) = monitor.workspaces.get_by_id(&previous_id) {
                previous.hide(false);
            }
            monitor.active_workspace().restore();

            log::trace!("Switched to workspace {workspace_id} on monitor {monitor_id}");
            Self::send(VirtualDesktopEvent::DesktopChanged {
                monitor: monitor_id.clone(),
                workspace: workspace_id.clone(),
            });
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

    /// Send a window to a specific workspace
    pub fn send_to(&self, window: &Window, workspace_id: &WorkspaceId) -> Result<()> {
        let window_id = window.address();
        let Some(monitor_id) = self.get_monitor_of_workspace(workspace_id) else {
            return Ok(());
        };

        // The whole move is done under the monitors lock, so it can't interleave with
        // other add/remove/send_to and leave the window on none or two workspaces.
        let moved = self.monitors.with_lock(|monitors| -> Result<bool> {
            // Only move windows that are already tracked on a workspace; non-interactable windows
            // don't belong to any workspace and pinned windows are not affected by workspaces,
            // so neither should be added to one by being "moved".
            let Some(current_workspace_id) = self.get_workspace_of_window(window_id) else {
                return Ok(false);
            };

            if &current_workspace_id == workspace_id {
                return Ok(false);
            }

            let target_monitor = monitors.get(&monitor_id).ok_or("Monitor not found")?;
            if !target_monitor.workspaces.contains(workspace_id) {
                return Err("Workspace not found in monitor".into());
            }
            let is_target_active = target_monitor.active_workspace_id() == workspace_id;

            // Remove window from current workspace
            for monitor in monitors.values_mut() {
                for row in monitor.workspaces.rows_mut() {
                    for workspace in row {
                        workspace.windows.retain(|w| w != &window_id);
                    }
                }
            }

            // Add window to target workspace, existence validated above
            if let Some(target_workspace) = monitors
                .get_mut(&monitor_id)
                .and_then(|m| m.workspaces.get_by_id_mut(workspace_id))
            {
                target_workspace.windows.push(window_id);
            }
            self.window_index.upsert(window_id, workspace_id.clone());

            // Hide window if target workspace is not active
            if !is_target_active {
                // Mark before minimizing, so the SystemMinimizeStart handlers (e.g. TWM)
                // already see it as hidden by the workspace and not by the user.
                let _ = MINIMIZED_BY_WORKSPACES.insert_sync(window_id);
                window.show_window(SW_MINIMIZE).ok();
            }

            Ok(true)
        })?;

        if moved {
            Self::send(VirtualDesktopEvent::WindowMoved {
                window: window_id,
                desktop: workspace_id.clone(),
            });
        }
        Ok(())
    }

    /// Create a new workspace column (or row) on a specific monitor
    pub fn create_desktop(&self, monitor_id: &MonitorId, as_row: bool) -> Result<WorkspaceId> {
        let workspace_id = self
            .monitors
            .get(monitor_id, |monitor| {
                let workspace_id = if as_row {
                    monitor.add_workspace_row()
                } else {
                    monitor.add_workspace_column()
                };
                // A new column/row adds a workspace on each row/column, index all of them.
                for workspace in monitor.workspaces.rows().iter().flatten() {
                    self.workspace_index
                        .upsert(workspace.id.clone(), monitor_id.clone());
                }
                workspace_id
            })
            .ok_or("Monitor not found")?;

        // Set wallpaper to the new workspace
        WorkspaceWallpapersManager::update_workspace_wallpapers_internal(self);

        Self::send(VirtualDesktopEvent::DesktopCreated(workspace_id.clone()));
        Ok(workspace_id)
    }

    /// Destroy a workspace, the owning monitor is resolved from the workspace index
    pub fn destroy_desktop(&self, workspace_id: &WorkspaceId) -> Result<()> {
        let monitor_id = self
            .get_monitor_of_workspace(workspace_id)
            .ok_or("Workspace not found")?;
        let (removed_ids, moved_windows, new_active) =
            self.monitors.with_lock(|monitors| -> Result<_> {
                let monitor = monitors.get_mut(&monitor_id).ok_or("Monitor not found")?;
                let previous_active = monitor.active_workspace_id().clone();
                let before: Vec<(WorkspaceId, Vec<isize>)> = monitor
                    .workspaces
                    .rows()
                    .iter()
                    .flatten()
                    .map(|w| (w.id.clone(), w.windows.clone()))
                    .collect();

                // Removing a column drops that column on every row, so more than one
                // workspace (maybe the active one) can go away; their windows are moved
                // to a sibling workspace.
                monitor.remove_workspace(workspace_id)?;

                let mut removed_ids = Vec::new();
                let mut moved_windows = Vec::new();
                for (id, windows) in before {
                    if monitor.workspaces.contains(&id) {
                        continue;
                    }
                    for window in windows {
                        if let Some(target) = monitor.workspaces.get_by_window_id(window) {
                            moved_windows.push((window, target.id.clone()));
                        }
                    }
                    removed_ids.push(id);
                }

                let active = monitor.active_workspace();
                let active_changed = active.id != previous_active;
                // Windows that landed on the active workspace coming from a hidden one
                // are still minimized, and if the active changed its own windows too.
                if active_changed || moved_windows.iter().any(|(_, target)| target == &active.id) {
                    active.restore();
                }

                let new_active = active_changed.then(|| active.id.clone());
                self.replace_indexes(monitors);
                Ok((removed_ids, moved_windows, new_active))
            })?;

        for (window, desktop) in moved_windows {
            Self::send(VirtualDesktopEvent::WindowMoved { window, desktop });
        }
        for id in removed_ids {
            Self::send(VirtualDesktopEvent::DesktopDestroyed(id));
        }
        if let Some(workspace) = new_active {
            Self::send(VirtualDesktopEvent::DesktopChanged {
                monitor: monitor_id,
                workspace,
            });
        }
        Ok(())
    }

    /// Rename a workspace, the owning monitor is resolved from the workspace index
    pub fn rename_desktop(&self, workspace_id: &WorkspaceId, name: Option<String>) -> Result<()> {
        let monitor_id = self
            .get_monitor_of_workspace(workspace_id)
            .ok_or("Workspace not found")?;
        self.monitors
            .get(&monitor_id, |monitor| {
                monitor.rename_workspace(workspace_id, name)
            })
            .ok_or("Monitor not found")??;
        // Not a DesktopChanged, the active workspace didn't change, only its data.
        Self::send(VirtualDesktopEvent::StateChanged);
        Ok(())
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
                let _ = MINIMIZED_BY_WORKSPACES.insert_sync(window.address());
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
            if is_minimized && !MINIMIZED_BY_WORKSPACES.contains_sync(addr) {
                continue;
            }

            if is_minimized {
                // Push before show_window to avoid a race where SystemMinimizeEnd
                // fires on the hook thread before this thread reaches the push.
                RESTORED_EVENT_QUEUE.push(*addr);
                // use normal show instead async cuz it will keep the order of restoring
                window.show_window(SW_RESTORE).log_error();
            }
            MINIMIZED_BY_WORKSPACES.remove_sync(addr);

            // ensure correct focus
            if idx == len - 1 {
                window.focus().log_error();
            }
        }
    }
}

impl VdManager {
    /// Update z-order: move the window to the end of its workspace's list,
    /// so it is the last restored (and focused) when the workspace is restored.
    fn move_to_top(&self, window_id: isize) {
        let mut updated = false;
        self.monitors.for_each(|(_, monitor)| {
            if let Some(workspace) = monitor.workspaces.get_by_window_id_mut(window_id) {
                workspace.windows.retain(|w| w != &window_id);
                workspace.windows.push(window_id);
                updated = true;
            }
        });
        if updated {
            bridge::request_save();
        }
    }

    /// Handles a window unminimized by the user (not by our workspace restore).
    fn on_user_unminimize(window: &Window) -> Result<()> {
        let window_id = window.address();
        let manager = Self::instance();

        let Ok(workspace_id) = window.workspace_id() else {
            // Add minimized windows during the scanning, to the current active workspace.
            // Pinned and non-interactable windows are filtered inside.
            manager.add_to_current_workspace(window);
            return Ok(());
        };

        let monitor_id = manager
            .get_monitor_of_workspace(&workspace_id)
            .ok_or("Workspace not found")?;

        if MonitorManager::instance()
            .get_cached_ids()
            .contains(&monitor_id)
        {
            // The foreground event of this window may arrive after this one, so
            // bring it to the top first, so the workspace restore ends focusing it.
            manager.move_to_top(window_id);
            // Restore workspace if the window was unminimized by the user via alt+tab or others
            manager.switch_to_id(&workspace_id)?;
        } else {
            // The workspace's monitor is disconnected, switching there would hide
            // windows that are now visible on other monitors. Instead only this
            // window is moved to the active workspace of the monitor it's shown on.
            MINIMIZED_BY_WORKSPACES.remove_sync(&window_id);
            let current_monitor_id = window.monitor().stable_id()?;
            if let Some(target_workspace_id) = manager
                .monitors
                .get(&current_monitor_id, |m| m.active_workspace_id().clone())
            {
                manager.send_to(window, &target_workspace_id)?;
            }
        }
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

                let result = Self::on_user_unminimize(&window);

                // Genuine user action (taskbar click, alt-tab, etc.), not an echo of our own
                // restore(). Let other modules (e.g. the TWM) react to it without having to
                // listen to the raw, indiscriminate SystemMinimizeEnd hook event themselves.
                // Always sent once the event is fully processed, so listeners find the window
                // on its final workspace (e.g. via `Window::workspace_id`).
                Self::send(VirtualDesktopEvent::WindowUnminimizedByUser { window: window_id });
                result?;
            }
            WinEvent::ObjectDestroy => {
                // A destroyed window will never emit the SystemMinimizeEnd that would
                // consume its entries, and its address could be reused by a new window.
                RESTORED_EVENT_QUEUE.retain(|w| w != &window_id);
                MINIMIZED_BY_WORKSPACES.remove_sync(&window_id);
            }
            WinEvent::SystemForeground | WinEvent::ObjectFocus => {
                Self::instance().move_to_top(window_id);
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
                        if monitor.workspaces.get_by_window_id(window_id).is_some() {
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
}
