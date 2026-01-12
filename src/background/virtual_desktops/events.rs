use seelen_core::{state::WorkspaceId, system_state::MonitorId};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum VirtualDesktopEvent {
    DesktopCreated(WorkspaceId),
    DesktopDestroyed(WorkspaceId),
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
}
