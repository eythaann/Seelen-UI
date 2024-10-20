use std::sync::atomic::Ordering;

use clap::Command;

use crate::{
    error_handler::Result, get_subcommands, hook::LOG_WIN_EVENTS, utils::TRACE_LOCK_ENABLED,
};

get_subcommands![
    /** Toggles the tracing of window events */
    ToggleWinEvents,
    /** Toggles the tracing of mutex lock */
    ToggleTraceLock,
];

pub struct CliDebugger;
impl CliDebugger {
    pub const CLI_IDENTIFIER: &'static str = "debugger";

    pub fn get_cli() -> Command {
        Command::new(Self::CLI_IDENTIFIER)
            .about("Debugger cli")
            .arg_required_else_help(true)
            .subcommands(SubCommand::commands())
    }

    pub fn process(matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        match subcommand {
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
