mod debugger;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use clap::{Arg, ArgAction, Command};
use debugger::CliDebugger;
use windows::Win32::System::Console::{AttachConsole, GetConsoleWindow, ATTACH_PARENT_PROCESS};

use crate::error_handler::Result;
use crate::modules::virtual_desk::{VirtualDesktopManager, VIRTUAL_DESKTOP_MANAGER};
use crate::seelen::{Seelen, SEELEN};
use crate::seelen_bar::FancyToolbar;
use crate::seelen_rofi::SeelenRofi;
use crate::seelen_weg::SeelenWeg;
use crate::seelen_wm_v2::instance::WindowManagerV2;
use crate::trace_lock;

use super::AppClient;

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
                        _ => Err("Unknown subcommand.".into()),
                    }
                } else {
                    Err("No subcommand was provided.".into())
                }
            }
        }
    }
}

pub fn get_app_command() -> Command {
    Command::new("Seelen UI")
        .author("eythaann")
        .about("Seelen Command Line Interface.")
        .long_about("Seelen Command Line Interface.")
        .before_help("")
        .after_help("To read more about Seelen visit https://github.com/eythaann/seelen-ui.git")
        .args([
            // we maintain this flag for backwards compatibility
            Arg::new("silent")
                .short('s')
                .long("silent")
                .action(ArgAction::SetTrue)
                .help("Unused flag"),
            Arg::new("verbose")
                .short('V')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Prints some extra process on the console."),
            Arg::new("version")
                .short('v')
                .long("version")
                .action(ArgAction::SetTrue)
                .help("Prints the current version of Seelen."),
            Arg::new("uri")
                .help("Path or URI to load.")
                .value_parser(clap::value_parser!(std::string::String))
                .action(clap::ArgAction::Set),
        ])
        .subcommands([
            Command::new("settings").about("Opens the Seelen settings gui."),
            VirtualDesktopManager::get_cli(),
            CliDebugger::get_cli(),
            FancyToolbar::get_cli(),
            WindowManagerV2::get_cli(),
            SeelenWeg::get_cli(),
            SeelenRofi::get_cli(),
        ])
}

// attach console could fail if not console to attach is present
pub fn attach_console() -> bool {
    let already_attached = unsafe { !GetConsoleWindow().is_invalid() };
    already_attached || unsafe { AttachConsole(ATTACH_PARENT_PROCESS).is_ok() }
}

/// Handles the CLI and will exit the process if needed.\
/// Performs redirection to the instance if needed too, will fail if no instance is running.
pub fn handle_console_cli() -> Result<()> {
    let matches = match get_app_command().try_get_matches() {
        Ok(m) => m,
        Err(e) => {
            // (help, --help or -h) and other sugestions are managed as error
            attach_console();
            e.exit();
        }
    };

    if matches.get_flag("silent") {
        crate::SILENT.store(true, Ordering::SeqCst);
    }

    if matches.get_flag("verbose") {
        crate::VERBOSE.store(true, Ordering::SeqCst);
    }

    if matches.get_flag("version") {
        attach_console();
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    if matches.subcommand().is_some() || matches.get_one::<String>("uri").is_some() {
        attach_console();
        AppClient::redirect_cli_to_instance()?;
        std::process::exit(0);
    }

    Ok(())
}

pub const URI: &str = "seelen-ui.uri:";

pub fn process_uri(uri: &str) -> Result<()> {
    log::trace!("Loading URI: {}", uri);

    let _contents = if uri.starts_with(URI) {
        uri.trim_start_matches(URI).to_string()
    } else {
        let path = PathBuf::from(uri);
        if path.is_file() && path.extension() == Some(OsStr::new("slu")) && path.exists() {
            std::fs::read_to_string(path)?
        } else {
            return Err("Invalid URI format".into());
        }
    };

    /* let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let decoded = engine.decode(contents.as_bytes())?;
    let resource: Resource = serde_yaml::from_slice(&decoded)?;
    FULL_STATE.rcu(|state| {
        let mut state = state.cloned();
        let _ = state.load_resource(resource.clone());
        state
    }); */
    Ok(())
}

pub fn handle_cli_events(matches: &clap::ArgMatches) -> Result<()> {
    if let Some(uri) = matches.get_one::<String>("uri") {
        return process_uri(uri).map_err(|e| format!("Corrupted SLU file: {}", e).into());
    }

    if let Some((subcommand, matches)) = matches.subcommand() {
        match subcommand {
            "settings" => {
                Seelen::show_settings()?;
            }
            VirtualDesktopManager::CLI_IDENTIFIER => {
                VIRTUAL_DESKTOP_MANAGER.load().process(matches)?;
            }
            CliDebugger::CLI_IDENTIFIER => {
                CliDebugger::process(matches)?;
            }
            WindowManagerV2::CLI_IDENTIFIER => {
                WindowManagerV2::process(matches)?;
            }
            FancyToolbar::CLI_IDENTIFIER => {
                let mut seelen = trace_lock!(SEELEN);
                for monitor in seelen.instances_mut() {
                    if let Some(toolbar) = monitor.toolbar_mut() {
                        toolbar.process(matches)?;
                    }
                }
            }
            SeelenWeg::CLI_IDENTIFIER => {
                SeelenWeg::process(matches)?;
                let mut seelen = trace_lock!(SEELEN);
                for monitor in seelen.instances_mut() {
                    if let Some(weg) = monitor.weg_mut() {
                        weg.process_by_instance(matches)?;
                    }
                }
            }
            SeelenRofi::CLI_IDENTIFIER => {
                if let Some(rofi) = trace_lock!(SEELEN).rofi_mut() {
                    rofi.process(matches)?;
                }
            }
            _ => {}
        }
        return Ok(());
    }
    Ok(())
}
