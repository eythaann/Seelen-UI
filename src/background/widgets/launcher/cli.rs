use serde::{Deserialize, Serialize};

use crate::error_handler::Result;

use super::SeelenRofi;
/// Seelen's App Launcher
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct AppLauncherCli {
    #[command(subcommand)]
    pub subcommand: LauncherSubCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum LauncherSubCommand {
    /// Shows/Hides the App Launcher
    Toggle,
}

impl SeelenRofi {
    pub fn process(&mut self, matches: AppLauncherCli) -> Result<()> {
        match matches.subcommand {
            LauncherSubCommand::Toggle => {
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
