mod art;
mod debugger;
mod uri;
mod win32;

use std::sync::atomic::Ordering;

use clap::Parser;
use debugger::DebuggerCli;
use slu_ipc::{messages::AppMessage, AppIpc};
use win32::Win32Cli;
use windows::Win32::System::Console::{AttachConsole, GetConsoleWindow, ATTACH_PARENT_PROCESS};

use crate::{
    app::SEELEN,
    cli::application::{art::ArtCli, uri::process_uri},
    error::Result,
    resources::cli::ResourceManagerCli,
    trace_lock,
    virtual_desktops::cli::VirtualDesktopCli,
    widgets::{
        launcher::cli::AppLauncherCli, popups::cli::PopupsCli, show_settings,
        task_switcher::cli::TaskSwitcherClient, weg::cli::WegCli,
        window_manager::cli::WindowManagerCli,
    },
};

/// Seelen Command Line Interface
#[derive(Debug, clap::Parser)]
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

#[derive(Debug, clap::Subcommand)]
pub enum AppCliCommand {
    /// Opens the Seelen settings gui.
    Settings,
    VirtualDesk(VirtualDesktopCli),
    Debugger(DebuggerCli),
    Launcher(AppLauncherCli),
    WindowManager(WindowManagerCli),
    Popup(PopupsCli),
    Weg(WegCli),
    Resource(ResourceManagerCli),
    Win32(Win32Cli),
    Art(ArtCli),
    TaskSwitcher(TaskSwitcherClient),
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

    if matches.should_be_redirected() {
        attach_console();
        matches.send_to_main_instance().await?;
        std::process::exit(0);
    }

    if matches.command.is_some() {
        matches.process()?;
        std::process::exit(0);
    }
    Ok(())
}

impl AppCli {
    pub fn should_be_redirected(&self) -> bool {
        if let Some(command) = &self.command {
            return !matches!(command, AppCliCommand::Win32(_) | AppCliCommand::Art(_));
        }
        self.uri.is_some()
    }

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

        AppIpc::send(AppMessage(args))
            .await
            .map_err(|_| "Can't stablish connection, ensure Seelen UI is running.")?;
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
                if let Some(rofi) = &mut trace_lock!(SEELEN).rofi {
                    rofi.process(command)?;
                }
            }
            AppCliCommand::WindowManager(command) => {
                command.process()?;
            }
            AppCliCommand::Weg(command) => {
                command.process()?;
            }
            AppCliCommand::Resource(command) => {
                command.process()?;
            }
            AppCliCommand::Popup(command) => {
                command.process()?;
            }
            AppCliCommand::TaskSwitcher(command) => {
                command.process()?;
            }
            // ========================================
            AppCliCommand::Win32(command) => {
                command.process()?;
            }
            AppCliCommand::Art(command) => {
                command.process();
            }
        }
        Ok(())
    }
}
