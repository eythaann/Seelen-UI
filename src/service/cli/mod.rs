pub mod processing;

use slu_ipc::{ServiceIpc, messages::SvcAction};

use clap::{Arg, ArgAction, Command};

use crate::{
    SERVICE_DISPLAY_NAME,
    error::Result,
    logger::SluServiceLogger,
    modules::{environment::EnvironmentManager, startup::StartupManager},
};

pub struct ServiceSubcommands;
impl ServiceSubcommands {
    pub const INSTALL: &str = "install";
    pub const UNINSTALL: &str = "uninstall";
    pub const STOP: &str = "stop";
}

pub fn get_cli() -> Command {
    Command::new(SERVICE_DISPLAY_NAME)
        .author("eythaann")
        .about("Seelen Command Line Interface.")
        .long_about("Seelen Command Line Interface.")
        .before_help("")
        .after_help("To read more about Seelen visit https://github.com/eythaann/seelen-ui.git")
        .subcommands([
            Command::new(ServiceSubcommands::INSTALL)
                .about("Installs or repairs the service (elevation required)."),
            Command::new(ServiceSubcommands::UNINSTALL)
                .about("Uninstalls the service (elevation required)."),
            Command::new(ServiceSubcommands::STOP).about("Stops the service."),
        ])
        .args([Arg::new("startup")
            .short('S')
            .long("startup")
            .action(ArgAction::SetTrue)
            .help("Indicates that the app was invoked from the start up action.")])
}

/// Handles the CLI and exits the process with 0 if it should
pub async fn handle_console_client() -> Result<()> {
    let matches = get_cli().get_matches();
    let subcommand = matches.subcommand();

    match subcommand {
        Some((ServiceSubcommands::INSTALL, _)) => {
            EnvironmentManager::instance().add_installation_dir_to_path()?;
            StartupManager::instance().register_service()?;
        }
        Some((ServiceSubcommands::UNINSTALL, _)) => {
            SluServiceLogger::uninstall_old_logging()?;
            EnvironmentManager::instance().remove_installation_dir_from_path()?;
            StartupManager::instance().unregister_service()?;
        }
        Some((ServiceSubcommands::STOP, _)) => {
            ServiceIpc::send(SvcAction::Stop).await?;
        }
        _ => {}
    }

    if subcommand.is_some() {
        std::process::exit(0);
    }
    Ok(())
}
