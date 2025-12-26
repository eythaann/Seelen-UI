pub mod processing;

use slu_ipc::{messages::SvcAction, ServiceIpc};

use clap::{Arg, ArgAction, Command};

use crate::{
    enviroment::{add_installation_dir_to_path, remove_installation_dir_from_path},
    error::Result,
    logger::SluServiceLogger,
    task_scheduler::TaskSchedulerHelper,
    SERVICE_DISPLAY_NAME,
};

pub struct ServiceSubcommands;
impl ServiceSubcommands {
    pub const INSTALL: &str = "install";
    pub const UNINSTALL: &str = "uninstall";
    pub const STOP: &str = "stop";
}

pub fn get_cli() -> Command {
    Command::new(SERVICE_DISPLAY_NAME.to_string())
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
            add_installation_dir_to_path()?;
            TaskSchedulerHelper::create_service_task()?;
        }
        Some((ServiceSubcommands::UNINSTALL, _)) => {
            SluServiceLogger::uninstall_old_logging()?;
            remove_installation_dir_from_path()?;
            TaskSchedulerHelper::remove_service_task()?;
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
