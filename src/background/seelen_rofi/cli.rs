use clap::Command;

use crate::{error_handler::Result, get_subcommands};

use super::SeelenRofi;

get_subcommands![
    /// Shows/Hides the App Launcher
    Toggle,
];

impl SeelenRofi {
    pub const CLI_IDENTIFIER: &'static str = "launcher";

    pub fn get_cli() -> Command {
        Command::new(Self::CLI_IDENTIFIER)
            .about("Seelen's App Launcher")
            .arg_required_else_help(true)
            .subcommands(SubCommand::commands())
    }

    pub fn process(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        match subcommand {
            SubCommand::Toggle => {
                if self.window.is_visible()? {
                    self.hide()?;
                } else {
                    self.show()?;
                }
            }
        };
        Ok(())
    }
}
