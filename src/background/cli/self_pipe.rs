use clap::Parser;
use slu_ipc::{
    messages::{AppMessage, IpcResponse},
    AppIpc,
};

use crate::{cli::application::AppCli, error::Result, modules::system_tray::SystemTrayManager};

pub struct SelfPipe;
impl SelfPipe {
    fn _handle_cli_message(mut argv: Vec<String>) -> Result<()> {
        if argv.is_empty() {
            return Ok(());
        }

        let first = argv.first().unwrap();
        if !first.contains("seelen-ui") {
            argv.insert(0, "seelen-ui.exe".to_string());
        }

        if let Ok(cli) = AppCli::try_parse_from(argv) {
            if let Err(err) = cli.process() {
                log::error!("Failed to process command: {err}");
                return Err(err);
            }
        }
        Ok(())
    }

    fn handle_message(message: AppMessage) -> IpcResponse {
        match message {
            AppMessage::Cli(argv) => {
                if let Err(err) = Self::_handle_cli_message(argv) {
                    return IpcResponse::Err(err.to_string());
                }
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
        AppIpc::send(AppMessage::Cli(vec!["settings".to_owned()])).await?;
        Ok(())
    }
}
