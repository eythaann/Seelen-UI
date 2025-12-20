use seelen_core::system_state::MonitorId;
use serde::{Deserialize, Serialize};

use crate::{error::Result, virtual_desktops::SluWorkspacesManager2, windows_api::window::Window};

/// Manage the Seelen Window Manager.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
#[command(alias = "vd")]
pub struct VirtualDesktopCli {
    #[command(subcommand)]
    pub subcommand: VdCommand,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, clap::Subcommand)]
pub enum VdCommand {
    /// Send the window to the specified workspace
    SendToWorkspace {
        /// The index of the workspace to switch to.
        index: usize,
    },
    /// Switch to the specified workspace
    SwitchWorkspace {
        /// The index of the workspace to switch to.
        index: usize,
    },
    /// Send the window to the specified workspace and switch to it
    MoveToWorkspace {
        /// The index of the workspace to switch to.
        index: usize,
    },
    /// Switch to the next workspace
    SwitchNext,
    /// Switch to the previous workspace
    SwitchPrev,
    /// Create a new workspace
    CreateNewWorkspace,
    /// Destroy the current workspace (will do nothing if there's only one workspace)
    DestroyCurrentWorkspace,
}

impl VirtualDesktopCli {
    pub fn process(self) -> Result<()> {
        self.subcommand.process()
    }
}

impl VdCommand {
    pub fn process(self) -> Result<()> {
        let focused_win = Window::get_foregrounded();
        let monitor_id: MonitorId = focused_win.monitor().stable_id()?.into();
        let vd = SluWorkspacesManager2::instance();

        match self {
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

                let next_idx = if self == VdCommand::SwitchNext {
                    (active_workspace_idx + 1) % len // next and cycle
                } else {
                    (active_workspace_idx + (len - 1)) % len // prev and cycle
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
}
