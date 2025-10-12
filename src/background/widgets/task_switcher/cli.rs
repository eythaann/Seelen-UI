use tauri::Emitter;

use crate::{app::get_app_handle, error::Result};

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
                log::trace!("Select next task");
                get_app_handle().emit("hidden::task-switcher-select-next", auto_confirm)?;
            }
            TaskSwitcherCommand::SelectPreviousTask { auto_confirm } => {
                get_app_handle().emit("hidden::task-switcher-select-previous", auto_confirm)?;
            }
        }
        Ok(())
    }
}
