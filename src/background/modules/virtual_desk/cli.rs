use std::sync::Arc;

use clap::Command;
use lazy_static::lazy_static;
use parking_lot::Mutex;

use crate::{error_handler::Result, get_subcommands, windows_api::WindowsApi};

use super::VirtualDesktopManager;

get_subcommands![
    /** Sends the window to the specified workspace */
    SendToWorkspace(index: usize => "The index of the workspace to switch to."),
    /** Switches to the specified workspace. */
    SwitchWorkspace(index: usize => "The index of the workspace to switch to."),
    /** Sends the window to the specified workspace and switches to it. */
    MoveToWorkspace(index: usize => "The index of the workspace to switch to."),
    /** Switch to the next workspace */
    SwitchNext,
    /** Switch to the previous workspace */
    SwitchPrev,
];

lazy_static! {
    static ref LOCKER: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

impl VirtualDesktopManager {
    pub const CLI_IDENTIFIER: &'static str = "virtual-desk";

    pub fn get_cli() -> Command {
        Command::new(Self::CLI_IDENTIFIER)
            .about("Manage the Seelen Window Manager.")
            .visible_alias("vd")
            .arg_required_else_help(true)
            .subcommands(SubCommand::commands())
    }

    pub fn process(&self, matches: &clap::ArgMatches) -> Result<()> {
        // Lock for the duration of the process to avoid concurrent switching of workspaces
        let _guard = LOCKER.lock();
        let subcommand = SubCommand::try_from(matches)?;
        match subcommand {
            SubCommand::SendToWorkspace(index) => {
                self.send_to(index, WindowsApi::get_foreground_window().0 as isize)?;
            }
            SubCommand::SwitchWorkspace(index) => {
                self.switch_to(index)?;
            }
            SubCommand::MoveToWorkspace(index) => {
                self.send_to(index, WindowsApi::get_foreground_window().0 as isize)?;
                std::thread::sleep(std::time::Duration::from_millis(20));
                self.switch_to(index)?;
            }
            _ => log::warn!("Unimplemented command: {:?}", subcommand),
        }
        Ok(())
    }
}
