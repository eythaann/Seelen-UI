use clap::{Args, Subcommand};
use seelen_core::state::WidgetTriggerPayload;

use crate::{error::Result, widgets::trigger_widget};

#[derive(Debug, Args)]
pub struct WidgetCli {
    #[command(subcommand)]
    command: WidgetCommand,
}

#[derive(Debug, Subcommand)]
enum WidgetCommand {
    /// Triggers a widget
    Trigger { widget_id: String },
}

impl WidgetCli {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            WidgetCommand::Trigger { widget_id } => {
                trigger_widget(WidgetTriggerPayload::new(widget_id.clone().into()))?;
            }
        }
        Ok(())
    }
}
