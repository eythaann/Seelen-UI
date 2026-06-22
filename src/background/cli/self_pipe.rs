use clap::Parser;
use slu_ipc::{
    commands::{AppCli, AppCommand},
    messages::{AppMessage, IpcResponse},
    AppIpc,
};

use crate::{
    cli::{process_app_command, uri::process_uri},
    error::{Result, ResultLogExt},
    modules::system_tray::SystemTrayManager,
};

pub struct SelfPipe;
impl SelfPipe {
    async fn handle_raw_cli_message(argv: Vec<String>) -> Result<()> {
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
            if let Err(err) = process_app_command(cli.command).await {
                log::error!("Failed to process command: {err}");
                return Err(err);
            }
        }
        Ok(())
    }

    async fn handle_message(message: AppMessage) -> IpcResponse {
        match message {
            AppMessage::Cli(argv) => {
                if let Err(err) = Self::handle_raw_cli_message(argv).await {
                    return IpcResponse::Err(err.to_string());
                }
            }
            AppMessage::Command(cmd) => {
                if let Err(err) = process_app_command(cmd).await {
                    log::error!("Failed to process command: {err}");
                    return IpcResponse::Err(err.to_string());
                }
            }
            AppMessage::OpenUri(uri) => {
                tokio::spawn(async move {
                    process_uri(&uri).await.log_error();
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
