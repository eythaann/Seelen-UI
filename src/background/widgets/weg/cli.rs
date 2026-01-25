use std::ops::Index;

use seelen_core::state::WegItem;
use serde::{Deserialize, Serialize};
use windows::Win32::UI::WindowsAndMessaging::SW_MINIMIZE;

use crate::{
    error::Result,
    trace_lock,
    widgets::weg::weg_items_impl::SEELEN_WEG_STATE,
    windows_api::{monitor::Monitor, window::Window, WindowsApi},
};

/// Seelen's dock commands
#[derive(Debug, Serialize, Deserialize, clap::Args)]
pub struct WegCli {
    #[command(subcommand)]
    pub subcommand: WegCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum WegCommand {
    /// Set foreground to the application which is idx-nth on the weg. If it is not started, then starts it.
    ForegroundOrRunApp {
        /// Which index should be started on weg.
        index: usize,
    },
}

impl WegCli {
    pub fn process(self) -> Result<()> {
        #[allow(irrefutable_let_patterns)]
        if let WegCommand::ForegroundOrRunApp { index } = self.subcommand {
            let id = Monitor::from(WindowsApi::monitor_from_cursor_point()).stable_id2()?;

            let items = trace_lock!(SEELEN_WEG_STATE).get_filtered_by_monitor()?;
            if let Some(wegitems) = items.get(&id) {
                let all_items: Vec<&WegItem> = wegitems
                    .left
                    .iter()
                    .chain(wegitems.center.iter())
                    .chain(wegitems.right.iter())
                    .filter(|item| matches!(item, WegItem::Pinned(_) | WegItem::Temporal(_)))
                    .collect();

                if all_items.len() <= index {
                    return Ok(());
                }

                let item = all_items.index(index);

                if let WegItem::Pinned(inner_data) | WegItem::Temporal(inner_data) = item {
                    if let Some(item) = inner_data.windows.first() {
                        let window = Window::from(item.handle);
                        if !window.is_window() {
                            return Ok(());
                        }

                        if window.is_focused() {
                            window.show_window_async(SW_MINIMIZE)?;
                        } else {
                            window.focus()?;
                        }
                    } else {
                        let program = inner_data.relaunch_program.clone();
                        let args = inner_data
                            .relaunch_args
                            .as_ref()
                            .map(|args| args.to_string());
                        let working_dir = inner_data.relaunch_in.clone();
                        WindowsApi::execute(program, args, working_dir, false)?;
                    }
                }
            }
        }
        Ok(())
    }
}
