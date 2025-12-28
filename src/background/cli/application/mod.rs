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

/// Defines how a CLI command should be executed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandExecutionMode {
    /// Command executes directly in the console that invokes it
    Direct,
    /// Command is sent to the main Seelen UI instance via IPC
    MainInstance,
}

/// Trait for CLI commands to define their execution mode
pub trait SluCliCommand {
    /// Returns the execution mode for this command.
    /// Default implementation returns MainInstance.
    fn execution_mode(&self) -> CommandExecutionMode {
        CommandExecutionMode::MainInstance
    }
}

/// Determines how the CLI invocation should be handled
#[derive(Debug)]
enum CliRoutingStrategy {
    /// Execute command directly in this process and exit (e.g., Win32 utils, Bundle, Translate)
    ExecuteDirect,
    /// Send command to main instance via IPC and exit (e.g., Settings, VirtualDesk, Load/Unload)
    /// The main instance will re-parse the args and execute the command
    RedirectToMainInstance,
    /// No command to execute, continue with normal app startup
    StartApp,
}

/// Seelen Command Line Interface
#[derive(Debug, clap::Parser)]
#[command(version, name = "Seelen UI")]
pub struct AppCli {
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

impl SluCliCommand for AppCliCommand {
    fn execution_mode(&self) -> CommandExecutionMode {
        match self {
            AppCliCommand::Win32(_) => CommandExecutionMode::Direct,
            AppCliCommand::Art(_) => CommandExecutionMode::Direct,
            AppCliCommand::Resource(r) => r.execution_mode(),
            // All other commands use the default MainInstance mode
            _ => CommandExecutionMode::MainInstance,
        }
    }
}

/// Main entry point for CLI handling.
///
/// Flow:
/// 1. Parse CLI arguments
/// 2. Configure global flags (startup, silent, verbose)
/// 3. Determine routing strategy based on command
/// 4. Execute according to strategy:
///    - ExecuteDirect: Run in this process and exit
///    - RedirectToMainInstance: Send to running instance via IPC and exit
///    - StartApp: Normal app startup, continue execution
pub async fn handle_console_client() -> Result<()> {
    let cli = parse_cli_args();
    configure_global_flags(&cli);

    // Determine how to handle this invocation
    let strategy = determine_routing_strategy(&cli);

    if cli.verbose {
        println!("Routing strategy: {strategy:?}");
    }

    // Execute according to strategy
    match strategy {
        CliRoutingStrategy::ExecuteDirect => {
            attach_console();
            cli.process_direct().await?;
            std::process::exit(0);
        }
        CliRoutingStrategy::RedirectToMainInstance => {
            attach_console();
            cli.send_to_main_instance().await?;
            std::process::exit(0);
        }
        CliRoutingStrategy::StartApp => {
            // Normal app startup, continue execution
            Ok(())
        }
    }
}

/// Parse CLI arguments, exits process on parse errors (--help, --version, etc.)
fn parse_cli_args() -> AppCli {
    match AppCli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            attach_console();
            e.exit();
        }
    }
}

/// Configure global flags based on CLI arguments
fn configure_global_flags(cli: &AppCli) {
    if cli.silent {
        crate::SILENT.store(true, Ordering::SeqCst);
    }

    if cli.verbose {
        crate::VERBOSE.store(true, Ordering::SeqCst);
        println!("Received args: {:#?}", std::env::args().collect::<Vec<_>>());
        println!("Parsed CLI: {cli:#?}");
    }
}

/// Determines how this CLI invocation should be routed
fn determine_routing_strategy(cli: &AppCli) -> CliRoutingStrategy {
    // If we have a command, check its execution mode
    if let Some(command) = &cli.command {
        match command.execution_mode() {
            CommandExecutionMode::Direct => {
                return CliRoutingStrategy::ExecuteDirect;
            }
            CommandExecutionMode::MainInstance => {
                return CliRoutingStrategy::RedirectToMainInstance;
            }
        }
    }

    // URIs are always redirected to main instance
    if cli.uri.is_some() {
        return CliRoutingStrategy::RedirectToMainInstance;
    }

    // No command or URI = normal app startup
    CliRoutingStrategy::StartApp
}

impl AppCli {
    /// Processes commands that execute directly in the console (async)
    pub async fn process_direct(self) -> Result<()> {
        match self.command {
            Some(cmd) => cmd.process_direct().await,
            None => Ok(()),
        }
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

        AppIpc::send(AppMessage::Cli(args))
            .await
            .map_err(|_| "Can't stablish connection, ensure Seelen UI is running.")?;
        Ok(())
    }
}

impl AppCliCommand {
    /// Processes commands that execute directly in console (async)
    pub async fn process_direct(self) -> Result<()> {
        match self {
            AppCliCommand::Win32(command) => {
                command.process_direct()?;
            }
            AppCliCommand::Art(command) => {
                command.process_direct();
            }
            AppCliCommand::Resource(command) => {
                command.process_direct().await?;
            }
            _ => {
                return Err("Command does not support direct execution".into());
            }
        }
        Ok(())
    }

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
            _ => {
                return Err("Command does not support instance execution".into());
            }
        }
        Ok(())
    }
}

// attach console could fail if not console to attach is present
pub fn attach_console() -> bool {
    let already_attached = unsafe { !GetConsoleWindow().is_invalid() };
    already_attached || unsafe { AttachConsole(ATTACH_PARENT_PROCESS).is_ok() }
}
