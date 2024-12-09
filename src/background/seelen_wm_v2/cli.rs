use clap::{Command, ValueEnum};
use seelen_core::handlers::SeelenEvent;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::error_handler::Result;
use crate::seelen::{get_app_handle, SEELEN};
use crate::state::application::FULL_STATE;
use crate::windows_api::window::Window;
use crate::windows_api::WindowsApi;
use crate::{get_subcommands, trace_lock};

use super::instance::WindowManagerV2;
use super::state::WM_STATE;

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum AllowedReservations {
    Left,
    Right,
    Top,
    Bottom,
    Stack,
    Float,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum AllowedFocus {
    Left,
    Right,
    Up,
    Down,
    Latest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Sizing {
    Increase,
    Decrease,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
pub enum Axis {
    Horizontal,
    Vertical,
    Top,
    Bottom,
    Left,
    Right,
}

get_subcommands![
    /** Open Dev Tools (only works if the app is running in dev mode) */
    Debug,
    /** Pause the Seelen Window Manager. */
    Pause,
    /** Resume the Seelen Window Manager. */
    Resume,
    /** Reserve space for a incoming window. */
    Reserve(side: AllowedReservations => "The position of the new window."),
    /** Cancels the current reservation */
    CancelReservation,
    /** Increases or decreases the size of the window */
    Width(action: Sizing => "What to do with the width."),
    /** Increases or decreases the size of the window */
    Height(action: Sizing => "What to do with the height."),
    /** Resets the size of the containers in current workspace to the default size. */
    ResetWorkspaceSize,
    /** Focuses the window in the specified position. */
    Focus(side: AllowedFocus => "The position of the window to focus."),
];

impl WindowManagerV2 {
    pub const CLI_IDENTIFIER: &'static str = "manager";

    pub fn get_cli() -> Command {
        Command::new(Self::CLI_IDENTIFIER)
            .about("Manage the Seelen Window Manager.")
            .visible_alias("wm")
            .arg_required_else_help(true)
            .subcommands(SubCommand::commands())
    }

    pub fn reserve(&self, _side: AllowedReservations) -> Result<()> {
        // self.emit(SeelenEvent::WMSetReservation, side)?;
        Ok(())
    }

    pub fn discard_reservation(&self) -> Result<()> {
        // self.emit(SeelenEvent::WMSetReservation, ())?;
        Ok(())
    }

    pub fn process(matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        match subcommand {
            SubCommand::Pause => {
                // self.pause(true, true)?;
            }
            SubCommand::Resume => {
                // self.pause(false, true)?;
                // Seelen::start_ahk_shortcuts()?;
            }
            SubCommand::Reserve(_side) => {
                // self.reserve(side)?;
            }
            SubCommand::CancelReservation => {
                // self.discard_reservation()?;
            }
            SubCommand::Debug =>
            {
                #[cfg(any(debug_assertions, feature = "devtools"))]
                if let Some(monitor) = trace_lock!(SEELEN).focused_monitor_mut() {
                    if let Some(wm) = monitor.wm() {
                        wm.window.open_devtools();
                    }
                }
            }
            SubCommand::Width(action) => {
                let foreground = Window::from(WindowsApi::get_foreground_window());
                let percentage = match action {
                    Sizing::Increase => FULL_STATE.load().settings.window_manager.resize_delta,
                    Sizing::Decrease => -FULL_STATE.load().settings.window_manager.resize_delta,
                };

                let state = trace_lock!(WM_STATE);
                let (m, w) = state.update_size(&foreground, Axis::Horizontal, percentage, false)?;
                get_app_handle().emit_to(
                    Self::get_label(&m.id),
                    SeelenEvent::WMSetLayout,
                    w.get_root_node(),
                )?;
            }
            SubCommand::Height(action) => {
                let foreground = Window::from(WindowsApi::get_foreground_window());
                let percentage = match action {
                    Sizing::Increase => FULL_STATE.load().settings.window_manager.resize_delta,
                    Sizing::Decrease => -FULL_STATE.load().settings.window_manager.resize_delta,
                };

                let state = trace_lock!(WM_STATE);
                let (m, w) = state.update_size(&foreground, Axis::Vertical, percentage, false)?;
                get_app_handle().emit_to(
                    Self::get_label(&m.id),
                    SeelenEvent::WMSetLayout,
                    w.get_root_node(),
                )?;
            }
            SubCommand::ResetWorkspaceSize => {
                // self.emit(SeelenEvent::WMResetWorkspaceSize, ())?;
            }
            SubCommand::Focus(_side) => {
                // self.emit(SeelenEvent::WMFocus, side)?;
            }
        };
        Ok(())
    }
}
