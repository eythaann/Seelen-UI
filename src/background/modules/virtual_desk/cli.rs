use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::{
    error_handler::Result, modules::virtual_desk::VirtualDesktopManagerTrait,
    windows_api::WindowsApi,
};

use super::VIRTUAL_DESKTOP_MANAGER;

lazy_static! {
    static ref LOCKER: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

/// Manage the Seelen Window Manager.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
#[command(alias = "vd")]
pub struct VirtualDesktopCli {
    #[command(subcommand)]
    pub subcommand: VdCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
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
        // Lock for the duration of the process to avoid concurrent switching of workspaces
        let _guard = LOCKER.lock();
        let vd = VIRTUAL_DESKTOP_MANAGER.load();
        match self {
            VdCommand::SendToWorkspace { index } => {
                vd.send_to(index, WindowsApi::get_foreground_window().0 as isize)?;
            }
            VdCommand::SwitchWorkspace { index } => {
                vd.switch_to(index)?;
            }
            VdCommand::MoveToWorkspace { index } => {
                vd.send_to(index, WindowsApi::get_foreground_window().0 as isize)?;
                std::thread::sleep(std::time::Duration::from_millis(20));
                vd.switch_to(index)?;
            }
            VdCommand::SwitchNext => {
                let idx = vd.get_current_idx()?;
                let len = vd.get_all()?.len();
                let next_idx = (idx + 1) % len; // next and cycle
                vd.switch_to(next_idx)?;
            }
            VdCommand::SwitchPrev => {
                let idx = vd.get_current_idx()?;
                let len = vd.get_all()?.len();
                let prev_idx = (idx + (len - 1)) % len; // prev and cycle
                vd.switch_to(prev_idx)?;
            }
            VdCommand::CreateNewWorkspace => {
                vd.create_desktop()?;
            }
            VdCommand::DestroyCurrentWorkspace => {
                vd.destroy_desktop(vd.get_current_idx()?)?;
            }
        }
        Ok(())
    }
}
