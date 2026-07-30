use slu_ipc::commands::VdCommand;
pub use slu_ipc::commands::VirtualDesktopCli;

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
fn cursor_monitor_id() -> Result<seelen_core::system_state::MonitorId> {
    let point = Mouse::get_cursor_pos()?;
    Monitor::from(&point).stable_id()
}

fn process_vd_command(cmd: VdCommand) -> Result<()> {
    let vd = SluWorkspacesManager2::instance();

    match cmd {
        VdCommand::SendToWorkspace { index } => {
            let focused_win = Window::get_foregrounded();
            let monitor_id = focused_win.monitor().stable_id()?;
            let workspace_id = vd
                .monitors
                .get(&monitor_id, |monitor| {
                    monitor
                        .active_row()
                        .get(index)
                        .map(|w| w.id.clone())
                        .ok_or_else(|| format!("Workspace index {} not found", index))
                })
                .ok_or("Monitor not found")??;
            vd.send_to(&focused_win, &workspace_id)?;
        }
        VdCommand::MoveToWorkspace { index } => {
            let focused_win = Window::get_foregrounded();
            let monitor_id = focused_win.monitor().stable_id()?;
            let workspace_id = vd
                .monitors
                .get(&monitor_id, |monitor| {
                    monitor
                        .active_row()
                        .get(index)
                        .map(|w| w.id.clone())
                        .ok_or_else(|| format!("Workspace index {} not found", index))
                })
                .ok_or("Monitor not found")??;
            vd.send_to(&focused_win, &workspace_id)?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            vd.switch_to_id(&monitor_id, &workspace_id)?;
        }
        VdCommand::SwitchWorkspace { index } => {
            let monitor_id = cursor_monitor_id()?;
            let workspace_id = vd
                .monitors
                .get(&monitor_id, |monitor| {
                    monitor
                        .active_row()
                        .get(index)
                        .map(|w| w.id.clone())
                        .ok_or_else(|| format!("Workspace index {} not found", index))
                })
                .ok_or("Monitor not found")??;
            vd.switch_to_id(&monitor_id, &workspace_id)?;
        }
        VdCommand::SwitchNext | VdCommand::SwitchPrev => {
            let monitor_id = cursor_monitor_id()?;

            let new_workspace_id = vd.monitors.with_lock(|monitors| {
                let monitor = monitors.get(&monitor_id).ok_or("Monitor not found")?;
                let (row, col) = monitor
                    .workspaces
                    .position(monitor.active_workspace_id())
                    .ok_or("Active workspace not found")?;

                let desired_col = if cmd == VdCommand::SwitchNext {
                    col + 1
                } else {
                    col - 1
                };

                Result::Ok(
                    monitor
                        .workspaces
                        .get(row, desired_col)
                        .map(|w| w.id.clone()),
                )
            })?;

            if let Some(new_workspace_id) = new_workspace_id {
                vd.switch_to_id(&monitor_id, &new_workspace_id)?;
            }
        }
        VdCommand::CreateNewWorkspace => {
            let monitor_id = cursor_monitor_id()?;
            let workspace_id = vd.create_desktop(&monitor_id)?;
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
