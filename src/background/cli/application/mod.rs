mod debugger;
pub(crate) mod uri;

pub use slu_ipc::commands::AppCli;
use slu_ipc::{commands::AppCommand, messages::AppMessage, AppIpc};

use std::sync::atomic::Ordering;

use crate::{
    error::Result,
    resources::cli as resources_cli,
    virtual_desktops::cli as vd_cli,
    widgets::{
        cli as widget_cli, popups::cli as popups_cli, show_settings,
        task_switcher::cli as task_switcher_cli, wallpaper_manager::cli as wallpaper_cli,
        weg::cli as weg_cli, window_manager::cli as wm_cli,
    },
};

/// WARNING: seelen-ui.exe CLI commands are deprecated.
///
/// Use `slu` instead.
#[derive(Debug, clap::Parser)]
#[command(version, name = "Seelen UI")]
struct MainCli {
    #[arg(long, default_value_t)]
    silent: bool,
    #[arg(long, default_value_t)]
    verbose: bool,
    /// Path or URI to open (e.g. from the Windows protocol handler).
    uri: Option<String>,
}

/// Called at startup by the main executable. If a URI is present it is forwarded
/// to the running instance via IPC and the process exits; otherwise returns Ok(())
/// so normal app startup continues.
pub async fn handle_console_client() -> Result<()> {
    use clap::Parser;
    let cli = match MainCli::try_parse() {
        Ok(cli) => cli,
        Err(e) => e.exit(),
    };

    if cli.silent {
        crate::SILENT.store(true, Ordering::SeqCst);
    }
    if cli.verbose {
        crate::VERBOSE.store(true, Ordering::SeqCst);
        println!("Received args: {:#?}", std::env::args().collect::<Vec<_>>());
        println!("Parsed CLI: {cli:#?}");
    }

    if let Some(uri) = cli.uri {
        AppIpc::send(AppMessage::OpenUri(uri))
            .await
            .map_err(|_| "Can't establish connection, ensure Seelen UI is running.")?;
        std::process::exit(0);
    }

    Ok(())
}

pub fn process_command(cmd: AppCommand) -> Result<()> {
    match cmd {
        AppCommand::Settings => {
            show_settings()?;
        }
        AppCommand::VirtualDesk(command) => {
            vd_cli::process(command)?;
        }
        AppCommand::Debugger(command) => {
            debugger::process(command)?;
        }
        AppCommand::WindowManager(command) => {
            wm_cli::process(command)?;
        }
        AppCommand::Weg(command) => {
            weg_cli::process(command)?;
        }
        AppCommand::Widget(command) => {
            widget_cli::run(command)?;
        }
        AppCommand::Resource(command) => {
            resources_cli::process(command)?;
        }
        AppCommand::Popup(command) => {
            popups_cli::process(command)?;
        }
        AppCommand::TaskSwitcher(command) => {
            task_switcher_cli::process(command)?;
        }
        AppCommand::Wallpaper(command) => {
            wallpaper_cli::process(command)?;
        }
        _ => {
            return Err("Command does not support instance execution".into());
        }
    }
    Ok(())
}
