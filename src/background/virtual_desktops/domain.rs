use std::collections::HashMap;

use seelen_core::{
    state::{VirtualDesktopMonitor, VirtualDesktops, WorkspaceId},
    system_state::MonitorId,
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum VirtualDesktopEvent {
    DesktopCreated(WorkspaceId),
    DesktopDestroyed(WorkspaceId),
    SwitchingDesktop(VirtualDesktops),
    /// Emitted once the workspace switch (and its settle grace period) is fully done.
    SwitchingFinished,
    DesktopChanged {
        monitor: MonitorId,
        workspace: WorkspaceId,
    },
    /// Emitted when the virtual desktops state changes (e.g., wallpapers updated)
    StateChanged,
    // DesktopNameChanged(WorkspaceId, String),
    /* DesktopMoved {
        desktop: WorkspaceId,
        old_index: usize,
        new_index: usize,
    }, */
    WindowAdded {
        window: isize,
        desktop: WorkspaceId,
    },
    /// Emitted when a window is or moved of virtual desktop.
    WindowMoved {
        window: isize,
        desktop: WorkspaceId,
    },
    WindowRemoved {
        window: isize,
    },
    /// A window's `Unminimized` that was a genuine user action (e.g. clicking a taskbar
    /// thumbnail or alt-tabbing to it), as opposed to an echo of `restore()` unminimizing the
    /// rest of a workspace's windows. Consumers that react to "the user brought this window
    /// back" (e.g. the TWM reactivating a stack tab) should listen for this instead of the raw
    /// `Unminimized` hook event, which fires for both cases indiscriminately.
    WindowUnminimizedByUser {
        window: isize,
    },
}

/// Builds the lookup indexes (workspace -> monitor, window -> workspace) of the given monitors.
pub fn get_indexes(
    monitors: &HashMap<MonitorId, VirtualDesktopMonitor>,
) -> (HashMap<WorkspaceId, MonitorId>, HashMap<isize, WorkspaceId>) {
    let mut workspace_index = HashMap::new();
    let mut window_index = HashMap::new();

    for (mid, m) in monitors {
        for row in m.workspaces.rows() {
            for space in row {
                workspace_index.insert(space.id.clone(), mid.clone());

                for window in &space.windows {
                    window_index.insert(*window, space.id.clone());
                }
            }
        }
    }

    (workspace_index, window_index)
}
