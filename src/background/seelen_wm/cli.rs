use clap::{Command, ValueEnum};
use seelen_core::handlers::SeelenEvent;
use seelen_core::state::VirtualDesktopStrategy;
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::HWND;

use crate::error_handler::Result;
use crate::get_subcommands;
use crate::modules::virtual_desk::get_vd_manager;
use crate::seelen::Seelen;
use crate::state::application::FULL_STATE;
use crate::windows_api::WindowsApi;

use super::WindowManager;

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

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum Sizing {
    Increase,
    Decrease,
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
    /** Switches to the specified workspace. */
    SwitchWorkspace(index: usize => "The index of the workspace to switch to."),
    /** Moves the window to the specified workspace. */
    MoveToWorkspace(index: usize => "The index of the workspace to switch to."),
    /** Sends the window to the specified workspace */
    SendToWorkspace(index: usize => "The index of the workspace to switch to."),
    /** Increases or decreases the size of the window */
    Height(action: Sizing => "What to do with the height."),
    /** Increases or decreases the size of the window */
    Width(action: Sizing => "What to do with the width."),
    /** Resets the size of the containers in current workspace to the default size. */
    ResetWorkspaceSize,
    /** Focuses the window in the specified position. */
    Focus(side: AllowedFocus => "The position of the window to focus."),
];

impl WindowManager {
    pub const CLI_IDENTIFIER: &'static str = "manager";

    pub fn get_cli() -> Command {
        Command::new(Self::CLI_IDENTIFIER)
            .about("Manage the Seelen Window Manager.")
            .visible_alias("wm")
            .arg_required_else_help(true)
            .subcommands(SubCommand::commands())
    }

    pub fn reserve(&self, side: AllowedReservations) -> Result<()> {
        self.emit(SeelenEvent::WMSetReservation, side)?;
        Ok(())
    }

    pub fn discard_reservation(&self) -> Result<()> {
        self.emit(SeelenEvent::WMSetReservation, ())?;
        Ok(())
    }

    pub fn process(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        match subcommand {
            SubCommand::Pause => {
                self.pause(true, true)?;
            }
            SubCommand::Resume => {
                self.pause(false, true)?;
                Seelen::start_ahk_shortcuts()?;
            }
            SubCommand::SwitchWorkspace(index) => {
                self.pseudo_pause()?;
                get_vd_manager().switch_to(index)?;
                if FULL_STATE.load().settings().virtual_desktop_strategy
                    == VirtualDesktopStrategy::Native
                {
                    if let Some(next) = Self::get_next_by_order(HWND(0)) {
                        WindowsApi::async_force_set_foreground(next);
                    }
                }
                self.pseudo_resume()?;
            }
            SubCommand::SendToWorkspace(index) => {
                let to_move = WindowsApi::get_foreground_window();
                get_vd_manager().send_to(index, to_move.0)?;
                if FULL_STATE.load().settings().virtual_desktop_strategy
                    == VirtualDesktopStrategy::Native
                {
                    if let Some(next) = Self::get_next_by_order(HWND(0)) {
                        WindowsApi::async_force_set_foreground(next);
                    }
                }
            }
            SubCommand::MoveToWorkspace(index) => {
                let to_move = WindowsApi::get_foreground_window();
                get_vd_manager().send_to(index, to_move.0)?;
                get_vd_manager().switch_to(index)?;
            }
            SubCommand::Reserve(side) => {
                self.reserve(side)?;
            }
            SubCommand::CancelReservation => {
                self.discard_reservation()?;
            }
            SubCommand::Debug => {
                #[cfg(any(debug_assertions, feature = "devtools"))]
                self.window.open_devtools();
            }
            SubCommand::Height(action) => {
                self.emit(SeelenEvent::WMUpdateHeight, action)?;
            }
            SubCommand::Width(action) => {
                self.emit(SeelenEvent::WMUpdateWidth, action)?;
            }
            SubCommand::ResetWorkspaceSize => {
                self.emit(SeelenEvent::WMResetWorkspaceSize, ())?;
            }
            SubCommand::Focus(side) => {
                self.emit(SeelenEvent::WMFocus, side)?;
            }
        };
        Ok(())
    }
}
