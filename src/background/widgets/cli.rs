pub use slu_ipc::commands::WidgetCli;
use slu_ipc::commands::WidgetCommand;

use seelen_core::state::WidgetTriggerPayload;

use crate::{error::Result, tauri_handlers::Handlers};

pub fn run(cmd: WidgetCli) -> Result<()> {
    match cmd.command {
        WidgetCommand::Trigger { widget_id } => {
            Handlers::trigger_widget(WidgetTriggerPayload::new(widget_id.into()))?;
        }
    }
    Ok(())
}
