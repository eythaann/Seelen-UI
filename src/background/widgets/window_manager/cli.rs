use clap::ValueEnum;
use seelen_core::handlers::SeelenEvent;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::app::get_app_handle;
use crate::error_handler::Result;
use crate::state::application::FULL_STATE;
use crate::trace_lock;
use crate::windows_api::window::Window;
use crate::windows_api::WindowsApi;

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

/// Manage the Seelen Window Manager.
#[derive(Debug, Serialize, Deserialize, clap::Args)]
#[command(alias = "wm")]
pub struct WindowManagerCli {
    #[command(subcommand)]
    pub subcommand: WmCommand,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum WmCommand {
    /// Open Dev Tools (only works if the app is running in dev mode)
    Debug,
    /// Pause the Seelen Window Manager.
    Pause,
    /// Resume the Seelen Window Manager.
    Resume,
    /// Reserve space for a incoming window.
    Reserve {
        /// The position of the new window.
        side: AllowedReservations,
    },
    /// Cancels the current reservation
    CancelReservation,
    /// Increases or decreases the size of the window
    Width {
        /// What to do with the width.
        action: Sizing,
    },
    /// Increases or decreases the size of the window
    Height {
        /// What to do with the height.
        action: Sizing,
    },
    /// Resets the size of the containers in current workspace to the default size.
    ResetWorkspaceSize,
    /// Focuses the window in the specified position.
    Focus {
        /// The position of the window to focus.
        side: AllowedFocus,
    },
}

impl WindowManagerCli {
    pub fn process(self) -> Result<()> {
        self.subcommand.process()
    }
}

impl WmCommand {
    pub fn process(self) -> Result<()> {
        match self {
            WmCommand::Pause => {
                // self.pause(true, true)?;
            }
            WmCommand::Resume => {
                // self.pause(false, true)?;
                // Seelen::start_ahk_shortcuts()?;
            }
            WmCommand::Reserve { .. } => {
                // self.reserve(side)?;
            }
            WmCommand::CancelReservation => {
                // self.discard_reservation()?;
            }
            WmCommand::Debug => {
                /* #[cfg(debug_assertions)]
                if let Some(monitor) = trace_lock!(crate::app::SEELEN).focused_monitor_mut() {
                    if let Some(wm) = monitor.wm() {
                        wm.window.open_devtools();
                    }
                } */
            }
            WmCommand::Width { action } => {
                let foreground = Window::from(WindowsApi::get_foreground_window());
                let percentage = match action {
                    Sizing::Increase => FULL_STATE.load().settings.by_widget.wm.resize_delta,
                    Sizing::Decrease => -FULL_STATE.load().settings.by_widget.wm.resize_delta,
                };

                let state = trace_lock!(WM_STATE);
                let (m, w) = state.update_size(&foreground, Axis::Horizontal, percentage, false)?;
                get_app_handle().emit_to(
                    WindowManagerV2::get_label(&m.id),
                    SeelenEvent::WMSetLayout,
                    w.get_root_node(),
                )?;
            }
            WmCommand::Height { action } => {
                let foreground = Window::from(WindowsApi::get_foreground_window());
                let percentage = match action {
                    Sizing::Increase => FULL_STATE.load().settings.by_widget.wm.resize_delta,
                    Sizing::Decrease => -FULL_STATE.load().settings.by_widget.wm.resize_delta,
                };

                let state = trace_lock!(WM_STATE);
                let (m, w) = state.update_size(&foreground, Axis::Vertical, percentage, false)?;
                get_app_handle().emit_to(
                    WindowManagerV2::get_label(&m.id),
                    SeelenEvent::WMSetLayout,
                    w.get_root_node(),
                )?;
            }
            WmCommand::ResetWorkspaceSize => {
                // self.emit(SeelenEvent::WMResetWorkspaceSize, ())?;
            }
            WmCommand::Focus { .. } => {
                // self.emit(SeelenEvent::WMFocus, side)?;
            }
        };
        Ok(())
    }
}
