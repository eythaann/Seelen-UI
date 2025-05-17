use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::{error_handler::Result, windows_api::WindowsApi};

use super::VIRTUAL_DESKTOP_MANAGER;

lazy_static! {
    static ref LOCKER: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

/// Manage the Seelen Window Manager.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
#[command(alias = "vd")]
pub struct VirtualDesktopCli {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
enum SubCommand {
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
}

impl VirtualDesktopCli {
    pub fn process(self) -> Result<()> {
        // Lock for the duration of the process to avoid concurrent switching of workspaces
        let _guard = LOCKER.lock();
        let vd = VIRTUAL_DESKTOP_MANAGER.load();
        match self.subcommand {
            SubCommand::SendToWorkspace { index } => {
                vd.send_to(index, WindowsApi::get_foreground_window().0 as isize)?;
            }
            SubCommand::SwitchWorkspace { index } => {
                vd.switch_to(index)?;
            }
            SubCommand::MoveToWorkspace { index } => {
                vd.send_to(index, WindowsApi::get_foreground_window().0 as isize)?;
                std::thread::sleep(std::time::Duration::from_millis(20));
                vd.switch_to(index)?;
            }
            _ => log::warn!("Unimplemented command: {:?}", self.subcommand),
        }
        Ok(())
    }
}
