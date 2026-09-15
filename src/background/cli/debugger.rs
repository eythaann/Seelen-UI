use std::sync::atomic::Ordering;

pub use slu_ipc::commands::DebuggerCli;

use crate::{error::Result, hook::LOG_WIN_EVENTS};

use slu_ipc::commands::DebuggerSubCommand;

pub fn process(cmd: DebuggerCli) -> Result<()> {
    match cmd.subcommand {
        DebuggerSubCommand::ToggleWinEvents => {
            LOG_WIN_EVENTS.store(!LOG_WIN_EVENTS.load(Ordering::Acquire), Ordering::Release);
        }
    };
    Ok(())
}
