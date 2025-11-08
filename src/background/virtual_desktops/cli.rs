use std::sync::LazyLock;

use parking_lot::Mutex;
use seelen_core::system_state::MonitorId;
use serde::{Deserialize, Serialize};

use crate::{error::Result, virtual_desktops::get_vd_manager, windows_api::window::Window};

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

static LOCKER: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

impl VdCommand {
    pub fn process(self) -> Result<()> {
        // we lock here to prevent concurrent calls
        let _lock = LOCKER.lock();

        let focused_win = Window::get_foregrounded();
        let monitor_id: MonitorId = focused_win.monitor().stable_id()?.into();
        let mut vd = get_vd_manager();

        match self {
            VdCommand::SendToWorkspace { index } => {
                vd.send_to(&monitor_id, index, &focused_win.address())?;
            }
            VdCommand::SwitchWorkspace { index } => {
                vd.switch_to(&monitor_id, index)?;
            }
            VdCommand::MoveToWorkspace { index } => {
                vd.send_to(&monitor_id, index, &focused_win.address())?;
                std::thread::sleep(std::time::Duration::from_millis(20));
                vd.switch_to(&monitor_id, index)?;
            }
            VdCommand::SwitchNext | VdCommand::SwitchPrev => {
                let active_monitor = vd.get_monitor_mut(&monitor_id);
                let active_workspace_idx = active_monitor
                    .workspaces
                    .iter()
                    .position(|w| w.id == active_monitor.current_workspace)
                    .ok_or("No active workspace")?;

                let len = active_monitor.workspaces.len();
                let next_idx = if self == VdCommand::SwitchNext {
                    (active_workspace_idx + 1) % len // next and cycle
                } else {
                    (active_workspace_idx + (len - 1)) % len // prev and cycle
                };
                vd.switch_to(&monitor_id, next_idx)?;
            }
            VdCommand::CreateNewWorkspace => {
                let workspace = vd.create_desktop(&monitor_id);
                vd.switch_to_id(&monitor_id, &workspace)?;
            }
            VdCommand::DestroyCurrentWorkspace => {
                let workspace_id = vd.get_monitor_mut(&monitor_id).current_workspace.clone();
                vd.destroy_desktop(&workspace_id);
            }
        }
        Ok(())
    }
}
