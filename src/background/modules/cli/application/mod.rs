mod debugger;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;

use base64::Engine;
use clap::{Arg, ArgAction, Command};
use debugger::CliDebugger;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::Win32::System::Console::{AttachConsole, FreeConsole, ATTACH_PARENT_PROCESS};

use crate::error_handler::Result;
use crate::seelen::{Seelen, SEELEN};
use crate::seelen_bar::FancyToolbar;
use crate::seelen_rofi::SeelenRofi;
use crate::seelen_weg::SeelenWeg;
use crate::seelen_wm::WindowManager;
use crate::state::application::FULL_STATE;
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
                    .long_help("Path or URI to load. (example: 'C:\\path\\to\\file.slu' or 'seelen-ui.uri:example')")
                    .value_parser(clap::value_parser!(std::string::String))
                    .action(clap::ArgAction::Set)
            ])
            .subcommands([
                Command::new("settings").about("Opens the Seelen settings gui."),
                CliDebugger::get_cli(),
                FancyToolbar::get_cli(),
                WindowManager::get_cli(),
                SeelenWeg::get_cli(),
                SeelenRofi::get_cli(),
            ])
    ));
}

pub fn attach_console() -> Result<()> {
    if !tauri::is_dev() {
        unsafe { AttachConsole(ATTACH_PARENT_PROCESS)? };
    }
    Ok(())
}

pub fn detach_console() -> Result<()> {
    if !tauri::is_dev() {
        unsafe { FreeConsole()? };
    }
    Ok(())
}

pub fn is_just_getting_info(matches: &clap::ArgMatches) -> Result<bool> {
    let mut r = false;

    if matches.get_flag("verbose") {
        attach_console()?;
        println!("{:?}", matches);
        detach_console()?;
    }

    if matches.get_flag("version") {
        attach_console()?;
        println!("{}", env!("CARGO_PKG_VERSION"));
        detach_console()?;
        r = true;
    }

    Ok(r)
}

const URI: &str = "seelen-ui.uri:";
const URI_MSIX: &str = "seelen-ui-msix.uri:";

pub fn process_uri(uri: &str) -> Result<()> {
    log::trace!("Loading URI: {}", uri);

    let contents = if uri.starts_with(URI) {
        uri.trim_start_matches(URI).to_string()
    } else if uri.starts_with(URI_MSIX) {
        uri.trim_start_matches(URI_MSIX).to_string()
    } else {
        let path = PathBuf::from(uri);
        if path.is_file() && path.extension() == Some(OsStr::new("slu")) && path.exists() {
            std::fs::read_to_string(path)?
        } else {
            return Err("Invalid URI format".into());
        }
    };

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let decoded = engine.decode(contents.as_bytes())?;

    let mut state = FULL_STATE.load().cloned();
    state.load_resource(serde_yaml::from_slice(&decoded)?)?;
    state.store();
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
            CliDebugger::CLI_IDENTIFIER => {
                CliDebugger::process(matches)?;
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
            SeelenWeg::CLI_IDENTIFIER => {
                let mut seelen = trace_lock!(SEELEN);
                for monitor in seelen.monitors_mut() {
                    if let Some(weg) = monitor.weg_mut() {
                        weg.process(matches)?;
                    }
                }
            }
            SeelenRofi::CLI_IDENTIFIER => {
                let mut seelen = trace_lock!(SEELEN);
                for monitor in seelen.monitors_mut() {
                    if let Some(rofi) = monitor.rofi_mut() {
                        rofi.process(matches)?;
                    }
                }
            }
            _ => {}
        }
        return Ok(());
    }

    Seelen::show_settings()?;
    Ok(())
}
