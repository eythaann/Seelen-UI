pub use slu_ipc::commands::PopupsCli;
use slu_ipc::commands::PopupsCommand;

use crate::{error::Result, widgets::popups::shortcut_registering::set_registering_shortcut};

pub fn process(cmd: PopupsCli) -> Result<()> {
    #[allow(clippy::single_match)]
    match cmd.subcommand {
        PopupsCommand::InternalSetShortcut { json } => {
            let shortcut: Option<Vec<String>> = serde_json::from_str(&json)?;
            set_registering_shortcut(shortcut)?;
        }
        _ => {}
    }
    Ok(())
}
