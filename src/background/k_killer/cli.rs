use clap::{value_parser, Arg, ArgAction, Command, ValueEnum};
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::error_handler::Result;
use crate::utils::{kebab_to_pascal, pascal_to_kebab};
use crate::windows_api::switch_desktop;

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
    Stack
}

get_subcommands![
    /** Pause the Seelen Window Manager. */
    Pause,
    /** Resume the Seelen Window Manager. */
    Resume,
    /** Reserve space for a incomming window. */
    Reserve(side: AllowedReservations => "The position of the new window."),
    /** Cancels the current reservation */
    CancelReservation,
    /** Switches to the specified workspace. */
    SwitchWorkspace(index: u8 => "The index of the workspace to switch to."),
    /** Open Dev Tools (only works if the app is running in dev mode) */
    Debug,
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

    pub fn process(&self, matches: &clap::ArgMatches) -> Result<()> {
        let subcommand = SubCommand::try_from(matches)?;
        log::trace!("Processing {:?}", subcommand);
        match subcommand {
            SubCommand::Pause => {
                println!("Paused");
            }
            SubCommand::Resume => {
                println!("Resume");
            }
            SubCommand::SwitchWorkspace(index) => {
                switch_desktop(index as u32);
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
        };
        Ok(())
    }
}
