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
    // DesktopNameChanged(WorkspaceId, String),
    /* DesktopMoved {
        desktop: WorkspaceId,
        old_index: usize,
        new_index: usize,
    }, */
    /// Emitted when a window is moved of the virtual desktop.
    WindowChanged {
        window: isize,
        desktop: WorkspaceId,
    },
}
