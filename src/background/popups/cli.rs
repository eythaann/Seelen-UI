use serde::{Deserialize, Serialize};

use crate::{error_handler::Result, popups::shortcut_registering::set_registering_shortcut};

/// Manage the Seelen Popups.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct PopupsCli {
    #[command(subcommand)]
    pub subcommand: PopupsCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum PopupsCommand {
    Create {
        /// json config
        config: String,
    },
    Update {
        /// id
        id: String,
        /// json config
        config: String,
    },
    Close {
        /// id
        id: String,
    },
    #[command(hide = true)]
    InternalSetShortcut { json: String },
}

impl PopupsCli {
    pub fn process(self) -> Result<()> {
        self.subcommand.process()
    }
}

impl PopupsCommand {
    pub fn process(self) -> Result<()> {
        #[allow(clippy::single_match)]
        match self {
            PopupsCommand::InternalSetShortcut { json } => {
                let shortcut: Option<Vec<String>> = serde_json::from_str(&json)?;
                set_registering_shortcut(shortcut)?;
            }
            _ => {}
        }
        Ok(())
    }
}
