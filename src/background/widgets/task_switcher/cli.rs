pub use slu_ipc::commands::TaskSwitcherClient;
use slu_ipc::commands::TaskSwitcherCommand;

use seelen_core::state::WidgetTriggerPayload;

use crate::{error::Result, widgets::trigger_widget};

pub fn process(cmd: TaskSwitcherClient) -> Result<()> {
    match cmd.command {
        TaskSwitcherCommand::SelectNextTask { auto_confirm } => {
            let mut args = WidgetTriggerPayload::new("@seelen/task-switcher".into());
            args.add_custom_arg("direction", "next");
            args.add_custom_arg("autoConfirm", auto_confirm);
            trigger_widget(args)?;
        }
        TaskSwitcherCommand::SelectPreviousTask { auto_confirm } => {
            let mut args = WidgetTriggerPayload::new("@seelen/task-switcher".into());
            args.add_custom_arg("direction", "previous");
            args.add_custom_arg("autoConfirm", auto_confirm);
            trigger_widget(args)?;
        }
    }
    Ok(())
}
