use std::env::args_os;
use std::sync::Arc;

use clap::Command;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::Wry;
use tauri::{AppHandle, Manager};

use crate::error_handler::Result;
use crate::seelen::SEELEN;

lazy_static! {
    pub static ref SEELEN_COMMAND_LINE: Arc<Mutex<Command>> = Arc::new(Mutex::new(
        Command::new("Seelen")
            .about("Seelen Command Line Interface.")
            .long_about("Seelen Command Line Interface.")
            .before_help("")
            .after_help("To read more about Seelen visit https://github.com/eythaann/seelen-ui.git")
            .args([
                clap::Arg::new("silent")
                    .short('s')
                    .long("silent")
                    .action(clap::ArgAction::SetTrue)
                    .help("Start only background processes."),
                clap::Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(clap::ArgAction::SetTrue)
                    .help("Prints some extra proccess on the console."),
                clap::Arg::new("version")
                    .short('V')
                    .long("version")
                    .action(clap::ArgAction::SetTrue)
                    .help("Prints the current version of Seelen."),
            ])
            .subcommands([
                clap::Command::new("settings").about("Opens the Seelen settings gui."),
                clap::Command::new("weg").about("Opens the Seelen Task Bar."),
                clap::Command::new("finder").about("Opens the Seelen Finder."),
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

    if args_os().any(|arg| arg == "--help" || arg == "-h") {
        return false;
    }

    true
}

pub fn handle_cli_events(app: &AppHandle<Wry>, matches: &clap::ArgMatches) -> Result<()> {
    if matches.get_flag("silent") {
        return Ok(());
    }

    if let Some((subcommand, _)) = matches.subcommand() {
        match subcommand {
            "settings" => {
                SEELEN.lock().show_settings()?;
            },
            _ => {}
        }
        return Ok(());
    }

    SEELEN.lock().show_settings()?;
    Ok(())
}
