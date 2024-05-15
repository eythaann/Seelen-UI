use clap::{Command, ValueEnum};
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::HWND;

use crate::error_handler::Result;
use crate::get_subcommands;
use crate::seelen::SEELEN;
use crate::utils::sleep_millis;
use crate::utils::virtual_desktop::VirtualDesktopManager;
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
            .subcommands(SubCommand::commands())
    }

    pub fn reserve(&self, side: AllowedReservations) -> Result<()> {
        self.emit("set-reservation", side)?;
        Ok(())
    }

    pub fn discard_reservation(&self) -> Result<()> {
        self.emit("set-reservation", ())?;
        Ok(())
    }

    pub fn process(&mut self, matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        log::trace!("Processing {:?}", subcommand);
        match subcommand {
            SubCommand::Pause => {
                self.pause(true, true)?;
            }
            SubCommand::Resume => {
                self.pause(false, true)?;
                SEELEN.lock().start_ahk_shortcuts()?;
            }
            SubCommand::SwitchWorkspace(index) => {
                let desktops = VirtualDesktopManager::enum_virtual_desktops()?;
                match desktops.get(index) {
                    Some(_) => {
                        self.pseudo_pause()?;
                        winvd::switch_desktop(index as u32)?;
                        sleep_millis(35); // to ensure avoid any artifacts
                        if let Some(next) = Self::get_next_by_order(HWND(0)) {
                            WindowsApi::force_set_foreground(next)?;
                        }
                        self.pseudo_resume()?;
                    }
                    None => log::error!("Invalid workspace index: {}", index),
                }
            }
            SubCommand::SendToWorkspace(index) => {
                let desktops = VirtualDesktopManager::enum_virtual_desktops()?;
                match desktops.get(index) {
                    Some(desktop) => {
                        let to_move = WindowsApi::get_foreground_window();
                        if self.is_managed(to_move) && !self.is_floating(to_move) {
                            self.emit_send_to_workspace(to_move, desktop.id())?;
                        }
                        let guid = desktop.guid();
                        std::thread::spawn(move || -> Result<()> {
                            winvd::move_window_to_desktop(guid, &to_move)?;
                            if let Some(next) = Self::get_next_by_order(to_move) {
                                WindowsApi::force_set_foreground(next)?;
                            }
                            Ok(())
                        });
                    }
                    None => log::error!("Invalid workspace index: {}", index),
                }
            }
            SubCommand::MoveToWorkspace(index) => {
                let desktops = VirtualDesktopManager::enum_virtual_desktops()?;
                match desktops.get(index) {
                    Some(desktop) => {
                        let to_move = WindowsApi::get_foreground_window();
                        let to_move_is_managed = self.is_managed(to_move);
                        self.pseudo_pause()?;
                        if to_move_is_managed && !self.is_floating(to_move) {
                            self.emit_send_to_workspace(to_move, desktop.id())?;
                        }
                        let guid = desktop.guid();
                        let desktop = winvd::Desktop::from(guid);
                        winvd::move_window_to_desktop(desktop, &to_move)?;
                        winvd::switch_desktop(desktop)?;
                        if to_move_is_managed {
                            self.pseudo_resume()?;
                        }
                    }
                    None => log::error!("Invalid workspace index: {}", index),
                }
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
                self.emit("update-height", action)?;
            }
            SubCommand::Width(action) => {
                self.emit("update-width", action)?;
            }
            SubCommand::ResetWorkspaceSize => {
                self.emit("reset-workspace-size", ())?;
            }
            SubCommand::Focus(side) => {
                self.emit("focus", side)?;
            }
        };
        Ok(())
    }
}
