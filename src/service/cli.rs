use std::{
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use clap::{Arg, Command};
use serde::Deserialize;

use crate::{
    error::Result, logger::SluServiceLogger, task_scheduler::TaskSchedulerHelper, SluService,
    SERVICE_DISPLAY_NAME, SLU_SERVICE,
};

pub struct ServiceSubcommands;
impl ServiceSubcommands {
    pub const INSTALL: &str = "install";
    pub const UNINSTALL: &str = "uninstall";
    pub const START: &str = "start";
    pub const STOP: &str = "stop";
    pub const SET_STARTUP: &str = "set-startup";
}

pub fn get_cli() -> Command {
    Command::new(SERVICE_DISPLAY_NAME.to_string())
        .author("eythaann")
        .about("Seelen Command Line Interface.")
        .long_about("Seelen Command Line Interface.")
        .before_help("")
        .after_help("To read more about Seelen visit https://github.com/eythaann/seelen-ui.git")
        .subcommands([
            Command::new(ServiceSubcommands::INSTALL).about("Installs or repairs the service."),
            Command::new(ServiceSubcommands::UNINSTALL).about("Uninstalls the service."),
            Command::new(ServiceSubcommands::START)
                .about("Starts the service (must be installed first)."),
            Command::new(ServiceSubcommands::STOP).about("Stops the service (TCP only)."),
            Command::new(ServiceSubcommands::SET_STARTUP)
                .about("Sets the Seelen UI App to start on boot.")
                .arg(
                    Arg::new("value")
                        .help("true or false")
                        .value_parser(clap::value_parser!(bool))
                        .action(clap::ArgAction::Set)
                        .required(true),
                ),
        ])
}

/// Handles the CLI and exits the process with 0 if it should
pub fn handle_cli() -> Result<()> {
    let mut should_stop_process = true;
    let matches = get_cli().get_matches();
    match matches.subcommand() {
        Some((ServiceSubcommands::INSTALL, _)) => {
            let result = SluService::install();
            if result.is_err() {
                SluService::uninstall()?;
                return result;
            }
            SluServiceLogger::install()?;
            TaskSchedulerHelper::create_service_task()?;
        }
        Some((ServiceSubcommands::UNINSTALL, _)) => {
            SluService::uninstall()?;
            SluServiceLogger::uninstall()?;
            TaskSchedulerHelper::remove_service_task()?;
        }
        Some((ServiceSubcommands::START, _)) => {
            if !SluService::is_running() {
                TaskSchedulerHelper::run_service_task()?;
            } else {
                println!("Service is already running.");
            }
        }
        Some((ServiceSubcommands::SET_STARTUP, arg)) => {
            let enabled: bool = *arg.get_one("value").unwrap();
            if enabled {
                TaskSchedulerHelper::create_app_startup_task()?;
            } else {
                TaskSchedulerHelper::remove_app_startup_task()?;
            }
        }
        _ => {
            should_stop_process = false;
        }
    }
    if should_stop_process {
        std::process::exit(0);
    }
    Ok(())
}

pub fn handle_tcp_cli(matches: &clap::ArgMatches) -> Result<()> {
    #[allow(clippy::single_match)]
    match matches.subcommand() {
        Some((ServiceSubcommands::STOP, _)) => {
            SLU_SERVICE.lock().stop();
        }
        _ => (),
    }
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Message {
    token: String,
    message: Vec<String>,
}

pub struct ServiceClient;
impl ServiceClient {
    fn token() -> &'static str {
        std::option_env!("SLU_SERVICE_CONNECTION_TOKEN").unwrap_or("__local__")
    }

    fn socket_path() -> PathBuf {
        PathBuf::from(r"C:\Windows\Temp\slu_service_tcp_socket")
    }

    fn handle_message(stream: TcpStream) -> Result<()> {
        let reader = std::io::BufReader::new(stream);
        let mut message: Message = serde_json::from_reader(reader)?;
        if message.token != Self::token() {
            return Ok(());
        }
        message.message.insert(0, "slu-service.exe".to_owned());
        if let Ok(matches) = get_cli().try_get_matches_from(message.message) {
            handle_tcp_cli(&matches)?;
        }
        Ok(())
    }

    pub fn listen_tcp() -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let socket_addr = listener.local_addr()?;
        let port = socket_addr.port();

        log::info!("TCP server listening on 127.0.0.1:{}", port);
        std::fs::write(Self::socket_path(), port.to_string())?;

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        if let Err(e) = Self::handle_message(stream) {
                            log::error!("Failed to handle message: {}", e);
                        }
                    }
                    Err(e) => log::error!("Failed to accept connection: {}", e),
                }
            }
        });
        Ok(())
    }

    pub fn connect_tcp() -> Result<TcpStream> {
        let port = std::fs::read_to_string(Self::socket_path())?;
        Ok(TcpStream::connect(format!("127.0.0.1:{}", port))?)
    }
}
