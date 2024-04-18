use std::env::args_os;
use std::sync::Arc;

use clap::{Arg, ArgAction, Command};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::AppHandle;
use tauri::Wry;

use crate::error_handler::Result;
use crate::k_killer::WindowManager;
use crate::seelen::SEELEN;

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
                    .help("Prints some extra proccess on the console."),
                Arg::new("version")
                    .short('V')
                    .long("version")
                    .action(ArgAction::SetTrue)
                    .help("Prints the current version of Seelen."),
            ])
            .subcommands([
                Command::new("settings").about("Opens the Seelen settings gui."),
                Command::new("weg").about("Opens the Seelen Task Bar."),
                Command::new("finder").about("Opens the Seelen Finder."),
                WindowManager::get_cli(),
            ])
    ));
}

type ShouldInitApp = bool;
pub fn handle_cli_info(matches: &clap::ArgMatches) -> ShouldInitApp {
    if matches.get_flag("verbose") {
        println!("{:?}", matches);
    }

    if matches.get_flag("version") {
        println!("{}", "1.0.0");
        return false;
    }

    if args_os().any(|arg| arg == "help" || arg == "--help" || arg == "-h") {
        return false;
    }

    true
}

pub fn handle_cli_events(_app: &AppHandle<Wry>, matches: &clap::ArgMatches) -> Result<()> {
    if let Some((subcommand, matches)) = matches.subcommand() {
        match subcommand {
            "settings" => {
                SEELEN.lock().show_settings()?;
            }
            WindowManager::CLI_IDENTIFIER => {
                if let Some(wm) = SEELEN.lock().wm() {
                    wm.process(matches)?;
                }
            }
            _ => {}
        }
        return Ok(());
    }

    SEELEN.lock().show_settings()?;
    Ok(())
}
