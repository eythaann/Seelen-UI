use clap::{value_parser, Arg, ArgAction, Command, ValueEnum};
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use windows::Win32::Foundation::HWND;

use crate::error_handler::Result;
use crate::seelen::SEELEN;
use crate::utils::virtual_desktop::VirtualDesktopManager;
use crate::utils::{kebab_to_pascal, pascal_to_kebab, sleep_millis};
use crate::windows_api::WindowsApi;

use super::WindowManager;

macro_rules! get_subcommands {
    ($(
        #[$meta:meta]
        $subcommand:ident $(($($arg_name:ident: $arg_type:ty => $arg_desc:literal),*))?,
    )*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        enum SubCommand {
            $(
                #[$meta]
                $subcommand$(($($arg_type),*))?,
            )*
        }

        impl SubCommand {
            pub fn commands() -> Vec<Command> {
                let mut commands = Vec::new();
                $(
                    commands.push({
                        let args: Vec<clap::Arg> = vec![
                            $($(
                                Arg::new(stringify!($arg_name))
                                    .help($arg_desc)
                                    .action(ArgAction::Set)
                                    .value_parser(value_parser!($arg_type))
                                    .required(true)
                            ),*)?
                        ];

                        let about = stringify!($meta).trim_start_matches("doc = r\"").trim_end_matches("\"").trim();
                        let command = pascal_to_kebab(stringify!($subcommand));
                        Command::new(command).about(about).args(args)
                    });
                )*
                commands
            }

            fn try_from(matches: &clap::ArgMatches) -> Result<Self> {
                if let Some((subcommand, matches)) = matches.subcommand() {
                    match kebab_to_pascal(subcommand).as_str() {
                        $(
                            stringify!($subcommand) => {
                                Ok(SubCommand::$subcommand$(($((matches.get_one(stringify!($arg_name)) as Option<&$arg_type>).unwrap().clone()),*))?)
                            },
                        )*
                        _ => Err(eyre!("Unknown subcommand.").into()),
                    }
                } else {
                    Err(eyre!("No subcommand was provided.").into())
                }
            }
        }
    }
}

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
    Lastest,
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
    /** Reserve space for a incomming window. */
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
        self.handle.emit_to(Self::TARGET, "set-reservation", side)?;
        Ok(())
    }

    pub fn discard_reservation(&self) -> Result<()> {
        self.handle.emit_to(Self::TARGET, "set-reservation", ())?;
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
                        std::thread::spawn(move || -> Result<()> {
                            winvd::switch_desktop(index as u32)?;
                            sleep_millis(35); // to ensure avoid any artifacs
                            let mut seelen = SEELEN.lock();
                            let wm = seelen.wm_mut().unwrap();
                            if let Some(next) = Self::get_next_by_order(HWND(0)) {
                                wm.force_focus(next)?;
                            }
                            wm.pseudo_resume()?;
                            Ok(())
                        });
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
                                SEELEN.lock().wm_mut().unwrap().force_focus(next)?;
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
                        std::thread::spawn(move || -> Result<()> {
                            let desktop = winvd::Desktop::from(guid);
                            winvd::move_window_to_desktop(desktop, &to_move)?;
                            winvd::switch_desktop(desktop)?;
                            if to_move_is_managed {
                                SEELEN.lock().wm_mut().unwrap().pseudo_resume()?;
                            }
                            Ok(())
                        });
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
                self.window.open_devtools();
            }
            SubCommand::Height(action) => {
                self.handle.emit_to(Self::TARGET, "update-height", action)?;
            }
            SubCommand::Width(action) => {
                self.handle.emit_to(Self::TARGET, "update-width", action)?;
            }
            SubCommand::ResetWorkspaceSize => {
                self.handle
                    .emit_to(Self::TARGET, "reset-workspace-size", ())?;
            }
            SubCommand::Focus(side) => {
                self.handle.emit_to(Self::TARGET, "focus", side)?;
            }
        };
        Ok(())
    }
}
