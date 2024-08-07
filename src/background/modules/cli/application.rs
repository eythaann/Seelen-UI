use std::env::args_os;
use std::sync::Arc;

use clap::{Arg, ArgAction, Command};
use lazy_static::lazy_static;
use parking_lot::Mutex;

use crate::error_handler::Result;
use crate::seelen::SEELEN;
use crate::seelen_bar::FancyToolbar;
use crate::seelen_wm::WindowManager;
use crate::trace_lock;

#[macro_export]
macro_rules! get_subcommands {
    ($(
        #[$meta:meta]
        $subcommand:ident $(($($arg_name:ident: $arg_type:ty => $arg_desc:literal),*))?,
    )*) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        enum SubCommand {
            $(
                #[$meta]
                $subcommand$(($($arg_type),*))?,
            )*
        }

        impl SubCommand {
            pub fn commands() -> Vec<clap::Command> {
                let mut commands = Vec::new();
                $(
                    commands.push({
                        let args: Vec<clap::Arg> = vec![
                            $($(
                                clap::Arg::new(stringify!($arg_name))
                                    .help($arg_desc)
                                    .action(clap::ArgAction::Set)
                                    .value_parser(clap::value_parser!($arg_type))
                                    .required(true)
                            ),*)?
                        ];

                        let about = stringify!($meta).trim_start_matches("doc = r\"").trim_end_matches("\"").trim();
                        let command = $crate::utils::pascal_to_kebab(stringify!($subcommand));
                        Command::new(command).about(about).args(args)
                    });
                )*
                commands
            }

            fn try_from(matches: &clap::ArgMatches) -> $crate::error_handler::Result<Self> {
                #[allow(unused_variables)]
                if let Some((subcommand, sub_matches)) = matches.subcommand() {
                    match $crate::utils::kebab_to_pascal(subcommand).as_str() {
                        $(
                            stringify!($subcommand) => {
                                Ok(SubCommand::$subcommand$(($((sub_matches.get_one(stringify!($arg_name)) as Option<&$arg_type>).unwrap().clone()),*))?)
                            },
                        )*
                        _ => Err(color_eyre::eyre::eyre!("Unknown subcommand.").into()),
                    }
                } else {
                    Err(color_eyre::eyre::eyre!("No subcommand was provided.").into())
                }
            }
        }
    }
}

lazy_static! {
    pub static ref SEELEN_COMMAND_LINE: Arc<Mutex<Command>> = Arc::new(Mutex::new(
        Command::new("Seelen")
            .author("eythaann")
            .about("Seelen Command Line Interface.")
            .long_about("Seelen Command Line Interface.")
            .before_help("")
            .after_help("To read more about Seelen visit https://github.com/eythaann/seelen-ui.git")
            .args([
                Arg::new("silent")
                    .short('s')
                    .long("silent")
                    .action(ArgAction::SetTrue)
                    .help("Start only background processes."),
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(ArgAction::SetTrue)
                    .help("Prints some extra process on the console."),
                Arg::new("version")
                    .short('V')
                    .long("version")
                    .action(ArgAction::SetTrue)
                    .help("Prints the current version of Seelen."),
            ])
            .subcommands([
                Command::new("settings").about("Opens the Seelen settings gui."),
                WindowManager::get_cli(),
                FancyToolbar::get_cli(),
                loader_command(),
            ])
    ));
}

pub fn loader_command() -> Command {
    let arg = Arg::new("value")
        .help("Value to load.")
        .required(true)
        .value_parser(clap::value_parser!(std::string::String))
        .action(clap::ArgAction::Set);

    Command::new("load")
        .about("Opens the Seelen Files or resolve URI.")
        .subcommands([
            Command::new("file")
                .about("Load a .slu file.")
                .arg(arg.clone().id("path")),
            Command::new("uri").about("Load a URI.").arg(arg.id("uri")),
        ])
}

pub fn is_just_getting_cmd_info(matches: &clap::ArgMatches) -> bool {
    if matches.get_flag("verbose") {
        println!("{:?}", matches);
    }

    if matches.get_flag("version") {
        println!("1.0.0");
        return true;
    }

    if args_os().any(|arg| arg == "help" || arg == "--help" || arg == "-h") {
        return true;
    }

    false
}

pub fn handle_cli_events(matches: &clap::ArgMatches) -> Result<()> {
    if let Some((subcommand, matches)) = matches.subcommand() {
        match subcommand {
            "settings" => {
                trace_lock!(SEELEN).show_settings()?;
            }
            WindowManager::CLI_IDENTIFIER => {
                if let Some(monitor) = trace_lock!(SEELEN).focused_monitor_mut() {
                    if let Some(wm) = monitor.wm_mut() {
                        wm.process(matches)?;
                    }
                }
            }
            FancyToolbar::CLI_IDENTIFIER => {
                let mut seelen = trace_lock!(SEELEN);
                for monitor in seelen.monitors_mut() {
                    if let Some(toolbar) = monitor.toolbar_mut() {
                        toolbar.process(matches)?;
                    }
                }
            }
            _ => {}
        }
        return Ok(());
    }

    trace_lock!(SEELEN).show_settings()?;
    Ok(())
}
