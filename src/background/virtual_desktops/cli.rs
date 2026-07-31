use seelen_core::{state::WorkspaceId, system_state::MonitorId};
pub use slu_ipc::commands::VirtualDesktopCli;
use slu_ipc::commands::{Direction, DirectionOrIndex, VdCommand};

use crate::{
    error::Result,
    virtual_desktops::SluWorkspacesManager2,
    windows_api::{input::Mouse, monitor::Monitor, window::Window},
};

pub fn process(cmd: VirtualDesktopCli) -> Result<()> {
    process_vd_command(cmd.subcommand)
}

/// Monitor targeted by cursor position, used by commands that don't act on a
/// specific window. Mirrors macOS' behavior of targeting the display under the
/// pointer instead of whichever window currently holds Win32 focus, since Windows
/// can reassign focus to another monitor when the active workspace becomes empty.
fn cursor_monitor_id() -> Result<MonitorId> {
    let point = Mouse::get_cursor_pos()?;
    Monitor::from(&point).stable_id()
}

/// Id of the workspace neighboring the given monitor's active workspace in the
/// grid, in the given direction. `None` if there is no workspace on that side.
fn neighbor_workspace_id(
    vd: &SluWorkspacesManager2,
    monitor_id: &MonitorId,
    direction: Direction,
) -> Result<Option<WorkspaceId>> {
    vd.monitors.with_lock(|monitors| {
        let monitor = monitors.get(monitor_id).ok_or("Monitor not found")?;
        let (row, col) = monitor
            .workspaces
            .position(monitor.active_workspace_id())
            .ok_or("Active workspace not found")?;

        let (desired_row, desired_col) = match direction {
            Direction::Left => (row, col.saturating_sub(1)),
            Direction::Right => (row, col + 1),
            Direction::Up => (row.saturating_sub(1), col),
            Direction::Down => (row + 1, col),
        };

        Result::Ok(
            monitor
                .workspaces
                .get(desired_row, desired_col)
                .map(|w| w.id.clone()),
        )
    })
}

/// Id of the workspace targeted by a `DirectionOrIndex`: either the neighbor in the
/// given direction, or the workspace at the given index in the active row.
fn target_workspace_id(
    vd: &SluWorkspacesManager2,
    monitor_id: &MonitorId,
    target: DirectionOrIndex,
) -> Result<Option<WorkspaceId>> {
    match target {
        DirectionOrIndex::Direction(direction) => neighbor_workspace_id(vd, monitor_id, direction),
        DirectionOrIndex::Index(index) => vd
            .monitors
            .get(monitor_id, |monitor| {
                monitor.active_row().get(index).map(|w| w.id.clone())
            })
            .ok_or_else(|| "Monitor not found".into()),
    }
}

fn process_vd_command(cmd: VdCommand) -> Result<()> {
    let vd = SluWorkspacesManager2::instance();

    match cmd {
        VdCommand::SwitchTo { direction } => {
            let monitor_id = cursor_monitor_id()?;
            if let Some(target_workspace_id) = target_workspace_id(vd, &monitor_id, direction)? {
                vd.switch_to_id(&monitor_id, &target_workspace_id)?;
            }
        }
        VdCommand::SendTo { direction } => {
            let focused_win = Window::get_foregrounded();
            let monitor_id = focused_win.monitor().stable_id()?;
            if let Some(target_workspace_id) = target_workspace_id(vd, &monitor_id, direction)? {
                vd.send_to(&focused_win, &target_workspace_id)?;
            }
        }
        VdCommand::MoveTo { direction } => {
            let focused_win = Window::get_foregrounded();
            let monitor_id = focused_win.monitor().stable_id()?;
            if let Some(target_workspace_id) = target_workspace_id(vd, &monitor_id, direction)? {
                vd.send_to(&focused_win, &target_workspace_id)?;
                std::thread::sleep(std::time::Duration::from_millis(20));
                vd.switch_to_id(&monitor_id, &target_workspace_id)?;
            }
        }
        VdCommand::CreateNewWorkspace => {
            let monitor_id = cursor_monitor_id()?;
            let workspace_id = vd.create_desktop(&monitor_id, false)?;
            vd.switch_to_id(&monitor_id, &workspace_id)?;
        }
        VdCommand::CreateNewWorkspaceRow => {
            let monitor_id = cursor_monitor_id()?;
            let workspace_id = vd.create_desktop(&monitor_id, true)?;
            vd.switch_to_id(&monitor_id, &workspace_id)?;
        }
        VdCommand::DestroyCurrentWorkspace => {
            let monitor_id = cursor_monitor_id()?;
            let workspace_id = vd
                .monitors
                .get(&monitor_id, |monitor| monitor.active_workspace_id().clone())
                .ok_or("Monitor not found")?;
            vd.destroy_desktop(&monitor_id, &workspace_id)?;
        }
    }
    Ok(())
}
