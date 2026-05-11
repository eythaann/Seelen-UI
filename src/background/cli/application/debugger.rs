use std::sync::atomic::Ordering;

pub use slu_ipc::commands::DebuggerCli;

use crate::{error::Result, hook::LOG_WIN_EVENTS, utils::TRACE_LOCK_ENABLED};

use slu_ipc::commands::DebuggerSubCommand;

pub fn process(cmd: DebuggerCli) -> Result<()> {
    match cmd.subcommand {
        DebuggerSubCommand::ToggleWinEvents => {
            LOG_WIN_EVENTS.store(!LOG_WIN_EVENTS.load(Ordering::Acquire), Ordering::Release);
        }
        DebuggerSubCommand::ToggleTraceLock => {
            TRACE_LOCK_ENABLED.store(
                !TRACE_LOCK_ENABLED.load(Ordering::Acquire),
                Ordering::Release,
            );
        }
    };
    Ok(())
}
