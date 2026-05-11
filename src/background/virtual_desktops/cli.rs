use slu_ipc::commands::VdCommand;
pub use slu_ipc::commands::VirtualDesktopCli;

use crate::{error::Result, virtual_desktops::SluWorkspacesManager2, windows_api::window::Window};

pub fn process(cmd: VirtualDesktopCli) -> Result<()> {
    process_vd_command(cmd.subcommand)
}

fn process_vd_command(cmd: VdCommand) -> Result<()> {
    let focused_win = Window::get_foregrounded();
    let monitor_id = focused_win.monitor().stable_id()?;
    let vd = SluWorkspacesManager2::instance();

    match cmd {
        VdCommand::SendToWorkspace { index } => {
            let workspace_id = vd
                .monitors
                .get(&monitor_id, |monitor| {
                    monitor
                        .workspaces
                        .get(index)
                        .map(|w| w.id.clone())
                        .ok_or_else(|| format!("Workspace index {} not found", index))
                })
                .ok_or("Monitor not found")??;
            vd.send_to(&focused_win, &workspace_id)?;
        }
        VdCommand::SwitchWorkspace { index } => {
            vd.switch_to(&monitor_id, index)?;
        }
        VdCommand::MoveToWorkspace { index } => {
            let workspace_id = vd
                .monitors
                .get(&monitor_id, |monitor| {
                    monitor
                        .workspaces
                        .get(index)
                        .map(|w| w.id.clone())
                        .ok_or_else(|| format!("Workspace index {} not found", index))
                })
                .ok_or("Monitor not found")??;
            vd.send_to(&focused_win, &workspace_id)?;
            std::thread::sleep(std::time::Duration::from_millis(20));
            vd.switch_to(&monitor_id, index)?;
        }
        VdCommand::SwitchNext | VdCommand::SwitchPrev => {
            let (active_workspace_idx, len) = vd
                .monitors
                .get(&monitor_id, |monitor| {
                    let active_workspace_idx = monitor
                        .workspaces
                        .iter()
                        .position(|w| &w.id == monitor.active_workspace_id())
                        .ok_or("No active workspace")?;
                    Result::Ok((active_workspace_idx, monitor.workspaces.len()))
                })
                .ok_or("Monitor not found")??;

            let next_idx = if cmd == VdCommand::SwitchNext {
                (active_workspace_idx + 1) % len
            } else {
                (active_workspace_idx + (len - 1)) % len
            };
            vd.switch_to(&monitor_id, next_idx)?;
        }
        VdCommand::CreateNewWorkspace => {
            let workspace_id = vd.create_desktop(&monitor_id)?;
            vd.switch_to_id(&monitor_id, &workspace_id)?;
        }
        VdCommand::DestroyCurrentWorkspace => {
            let workspace_id = vd
                .monitors
                .get(&monitor_id, |monitor| monitor.active_workspace_id().clone())
                .ok_or("Monitor not found")?;
            vd.destroy_desktop(&monitor_id, &workspace_id)?;
        }
    }
    Ok(())
}
