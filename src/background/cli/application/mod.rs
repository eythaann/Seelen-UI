mod debugger;
mod resources;
mod uri;
mod win32;

use std::sync::atomic::Ordering;

use clap::Parser;
use debugger::DebuggerCli;
use resources::WidgetCli;
use serde::{Deserialize, Serialize};
use slu_ipc::AppIpc;
use win32::Win32Cli;
use windows::Win32::System::Console::{AttachConsole, GetConsoleWindow, ATTACH_PARENT_PROCESS};

use crate::{
    cli::application::uri::process_uri, error_handler::Result, popups::cli::PopupsCli,
    seelen::SEELEN, seelen_rofi::cli::AppLauncherCli, seelen_weg::cli::WegCli,
    seelen_wm_v2::cli::WindowManagerCli, trace_lock, virtual_desktops::cli::VirtualDesktopCli,
    widgets::show_settings,
};

/// Seelen Command Line Interface
#[derive(Debug, Serialize, Deserialize, clap::Parser)]
#[command(version, name = "Seelen UI")]
pub struct AppCli {
    /// Indicates that the app was invoked from the start up action.
    #[arg(long, default_value_t)]
    startup: bool,
    /// Unused flag
    #[arg(long, default_value_t)]
    silent: bool,
    /// Prints some extra information on the console.
    #[arg(long, default_value_t)]
    verbose: bool,
    /// Path or URI to load.
    uri: Option<String>,
    #[command(subcommand)]
    command: Option<AppCliCommand>,
}

#[derive(Debug, Serialize, Deserialize, clap::Subcommand)]
pub enum AppCliCommand {
    /// Opens the Seelen settings gui.
    Settings,
    VirtualDesk(VirtualDesktopCli),
    Debugger(DebuggerCli),
    Launcher(AppLauncherCli),
    WindowManager(WindowManagerCli),
    Popup(PopupsCli),
    Weg(WegCli),
    Widget(WidgetCli),
    Win32(Win32Cli),
}

// attach console could fail if not console to attach is present
pub fn attach_console() -> bool {
    let already_attached = unsafe { !GetConsoleWindow().is_invalid() };
    already_attached || unsafe { AttachConsole(ATTACH_PARENT_PROCESS).is_ok() }
}

/// Handles the CLI and will exit the process if needed.\
/// Performs redirection to the instance if needed too, will fail if no instance is running.
pub async fn handle_console_client() -> Result<()> {
    let matches = match AppCli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // (help, --help or -h) and other sugestions are managed as error
            attach_console();
            e.exit();
        }
    };

    if matches.startup {
        crate::STARTUP.store(true, Ordering::SeqCst);
    }

    if matches.silent {
        crate::SILENT.store(true, Ordering::SeqCst);
    }

    if matches.verbose {
        crate::VERBOSE.store(true, Ordering::SeqCst);
        println!("Received {:#?}", std::env::args());
        println!("Parsed {matches:#?}");
    }

    // win32 commands are handled separately, as this should run on a win32 context.
    if let Some(AppCliCommand::Win32(ctx)) = &matches.command {
        ctx.process()?;
        std::process::exit(0);
    }

    if matches.command.is_some() || matches.uri.is_some() {
        attach_console();
        matches.send_to_main_instance().await?;
        std::process::exit(0);
    }

    Ok(())
}

impl AppCli {
    /// intended to be called on the main instance
    pub fn process(self) -> Result<()> {
        if let Some(uri) = self.uri {
            return process_uri(&uri);
        }
        match self.command {
            Some(cmd) => cmd.process(),
            None => Ok(()),
        }
    }

    /// will fail if no instance is running
    pub async fn send_to_main_instance(self) -> Result<()> {
        let mut args = Vec::new();
        let working_dir = std::env::current_dir()?;

        for arg in std::env::args() {
            if arg.starts_with("./")
                || arg.starts_with(".\\")
                || arg.starts_with("../")
                || arg.starts_with("..\\")
            {
                args.push(working_dir.join(arg).to_string_lossy().to_string());
                continue;
            }
            args.push(arg);
        }

        if self.verbose {
            println!("Sending {args:#?}");
        }

        AppIpc::send(args).await?;
        Ok(())
    }
}

impl AppCliCommand {
    pub fn process(self) -> Result<()> {
        match self {
            AppCliCommand::Settings => {
                show_settings()?;
            }
            AppCliCommand::VirtualDesk(command) => {
                command.process()?;
            }
            AppCliCommand::Debugger(command) => {
                command.process()?;
            }
            AppCliCommand::Launcher(command) => {
                if let Some(rofi) = trace_lock!(SEELEN).rofi_mut() {
                    rofi.process(command)?;
                }
            }
            AppCliCommand::WindowManager(command) => {
                command.process()?;
            }
            AppCliCommand::Weg(command) => {
                command.process()?;
            }
            AppCliCommand::Widget(command) => {
                command.process()?;
            }
            AppCliCommand::Popup(command) => {
                command.process()?;
            }
            AppCliCommand::Win32(_) => {}
        }
        Ok(())
    }
}
