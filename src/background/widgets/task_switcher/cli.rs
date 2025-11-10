use seelen_core::state::WidgetTriggerPayload;

use crate::{error::Result, widgets::trigger_widget};

#[derive(Debug, clap::Args)]
pub struct TaskSwitcherClient {
    #[command(subcommand)]
    command: TaskSwitcherCommand,
}

impl TaskSwitcherClient {
    pub fn process(self) -> Result<()> {
        self.command.process()
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskSwitcherCommand {
    SelectNextTask {
        #[clap(long)]
        auto_confirm: bool,
    },
    SelectPreviousTask {
        #[clap(long)]
        auto_confirm: bool,
    },
}

impl TaskSwitcherCommand {
    pub fn process(self) -> Result<()> {
        match self {
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
}
