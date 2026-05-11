use clap::Parser;
use slu_ipc::{
    commands::AppCommand,
    messages::{AppMessage, IpcResponse},
    AppIpc,
};

use crate::{
    cli::application::{self, uri::process_uri, AppCli},
    error::{Result, ResultLogExt},
    modules::system_tray::SystemTrayManager,
};

pub struct SelfPipe;
impl SelfPipe {
    fn handle_raw_cli_message(argv: Vec<String>) -> Result<()> {
        if argv.is_empty() {
            return Ok(());
        }

        // Normalize argv: always use a fixed program name as argv[0] for clap.
        // The first element may be an executable path (seelen-ui.exe, slu.exe, etc.)
        // or already a subcommand when called internally.
        let normalized: Vec<String> =
            if argv[0].ends_with(".exe") || argv[0].contains('\\') || argv[0].contains('/') {
                std::iter::once("seelen-ui.exe".to_string())
                    .chain(argv.into_iter().skip(1))
                    .collect()
            } else {
                std::iter::once("seelen-ui.exe".to_string())
                    .chain(argv)
                    .collect()
            };

        if let Ok(cli) = AppCli::try_parse_from(normalized) {
            if let Err(err) = application::process_command(cli.command) {
                log::error!("Failed to process command: {err}");
                return Err(err);
            }
        }
        Ok(())
    }

    fn handle_message(message: AppMessage) -> IpcResponse {
        match message {
            AppMessage::Cli(argv) => {
                if let Err(err) = Self::handle_raw_cli_message(argv) {
                    return IpcResponse::Err(err.to_string());
                }
            }
            AppMessage::Command(cmd) => {
                if let Err(err) = application::process_command(cmd) {
                    log::error!("Failed to process command: {err}");
                    return IpcResponse::Err(err.to_string());
                }
            }
            AppMessage::OpenUri(uri) => {
                std::thread::spawn(move || {
                    process_uri(&uri).log_error();
                });
            }
            AppMessage::TrayChanged(event) => {
                SystemTrayManager::handle_tray_event(event);
            }
            AppMessage::Debug(_msg) => {}
        }

        IpcResponse::Success
    }

    pub fn start_listener() -> Result<()> {
        AppIpc::start(Self::handle_message)?;
        Ok(())
    }

    pub async fn request_open_settings() -> Result<()> {
        AppIpc::send(AppMessage::Command(AppCommand::Settings)).await?;
        Ok(())
    }
}
