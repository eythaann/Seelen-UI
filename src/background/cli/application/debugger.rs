use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};

use crate::{error_handler::Result, hook::LOG_WIN_EVENTS, utils::TRACE_LOCK_ENABLED};

/// Debugger cli
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct DebuggerCli {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
enum SubCommand {
    /// Toggles the tracing of window events
    ToggleWinEvents,
    /// Toggles the tracing of mutex lock
    ToggleTraceLock,
}

impl DebuggerCli {
    pub fn process(self) -> Result<()> {
        match self.subcommand {
            SubCommand::ToggleWinEvents => {
                LOG_WIN_EVENTS.store(!LOG_WIN_EVENTS.load(Ordering::Acquire), Ordering::Release);
            }
            SubCommand::ToggleTraceLock => {
                TRACE_LOCK_ENABLED.store(
                    !TRACE_LOCK_ENABLED.load(Ordering::Acquire),
                    Ordering::Release,
                );
            }
        };
        Ok(())
    }
}
