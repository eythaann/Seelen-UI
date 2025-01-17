use std::{
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use clap::{Arg, Command};
use serde::Deserialize;

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
            Command::new(ServiceSubcommands::INSTALL)
                .about("Installs or repairs the service (elevation required)."),
            Command::new(ServiceSubcommands::UNINSTALL)
                .about("Uninstalls the service (elevation required)."),
            Command::new(ServiceSubcommands::STOP).about("Stops the service (TCP)."),
            Command::new(ServiceSubcommands::SET_STARTUP)
                .about("Sets the service to start on boot (TCP).")
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
    let matches = get_cli().get_matches();
    let subcommand = matches.subcommand();
    match subcommand {
        Some((ServiceSubcommands::INSTALL, _)) => {
            SluServiceLogger::install()?;
            add_installation_dir_to_path()?;
            TaskSchedulerHelper::create_service_task()?;
        }
        Some((ServiceSubcommands::UNINSTALL, _)) => {
            SluServiceLogger::uninstall()?;
            remove_installation_dir_from_path()?;
            TaskSchedulerHelper::remove_service_task()?;
        }
        Some((ServiceSubcommands::STOP, _)) => {
            ServiceClient::emit_stop_signal()?;
        }
        _ => {}
    }
    if subcommand.is_some() {
        std::process::exit(0);
    }
    Ok(())
}

pub fn handle_tcp_cli(matches: &clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some((ServiceSubcommands::STOP, _)) => {
            crate::stop();
        }
        Some((ServiceSubcommands::SET_STARTUP, arg)) => {
            let enabled: bool = *arg.get_one("value").unwrap();
            TaskSchedulerHelper::set_enabled_service_task(enabled)?;
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
        log::trace!("CLI command received: {}", message.message.join(" "));
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

    fn send_message(message: &[&str]) -> Result<()> {
        let stream = Self::connect_tcp()?;
        let writter = std::io::BufWriter::new(stream);
        serde_json::to_writer(
            writter,
            &serde_json::json!({
                "token": Self::token(),
                "message": message,
            }),
        )?;
        Ok(())
    }

    pub fn emit_stop_signal() -> Result<()> {
        Self::send_message(&["stop"])
    }
}
