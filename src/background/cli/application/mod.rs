mod debugger;
mod resources;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

use clap::Parser;
use debugger::DebuggerCli;
use itertools::Itertools;
use resources::WidgetCli;
use seelen_core::resource::{ResourceKind, SluResourceFile};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use windows::Win32::System::Console::{AttachConsole, GetConsoleWindow, ATTACH_PARENT_PROCESS};

use crate::{
    cli::SelfPipe,
    error_handler::Result,
    modules::virtual_desk::cli::VirtualDesktopCli,
    popups::POPUPS_MANAGER,
    seelen::{Seelen, SEELEN},
    seelen_rofi::cli::AppLauncherCli,
    seelen_weg::cli::WegCli,
    seelen_wm_v2::cli::WindowManagerCli,
    trace_lock,
    utils::constants::SEELEN_COMMON,
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
    Weg(WegCli),
    Widget(WidgetCli),
}

// attach console could fail if not console to attach is present
pub fn attach_console() -> bool {
    let already_attached = unsafe { !GetConsoleWindow().is_invalid() };
    already_attached || unsafe { AttachConsole(ATTACH_PARENT_PROCESS).is_ok() }
}

/// Handles the CLI and will exit the process if needed.\
/// Performs redirection to the instance if needed too, will fail if no instance is running.
pub fn handle_console_client() -> Result<()> {
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

    if matches.command.is_some() || matches.uri.is_some() {
        attach_console();
        matches.send_to_main_instance()?;
        std::process::exit(0);
    }

    Ok(())
}

fn path_by_resource_kind(kind: &ResourceKind) -> &Path {
    match kind {
        ResourceKind::Theme => SEELEN_COMMON.user_themes_path(),
        ResourceKind::IconPack => SEELEN_COMMON.user_icons_path(),
        ResourceKind::Widget => SEELEN_COMMON.user_widgets_path(),
        ResourceKind::Plugin => SEELEN_COMMON.user_plugins_path(),
        ResourceKind::Wallpaper => SEELEN_COMMON.user_wallpapers_path(),
        ResourceKind::SoundPack => SEELEN_COMMON.user_sounds_path(),
    }
}

impl AppCli {
    pub const URI: &str = "seelen-ui.uri:";

    pub fn process_uri(uri: &str) -> Result<()> {
        log::trace!("Loading URI: {}", uri);

        if !uri.starts_with(Self::URI) {
            let path = PathBuf::from(uri);
            if !path.is_file() || path.extension() != Some(OsStr::new("slu")) || !path.exists() {
                return Err("Invalid file to load".into());
            }

            let file = SluResourceFile::load(&path)?;
            let path_to_store = path_by_resource_kind(&file.resource.kind)
                .join(format!("{}.slu", file.resource.id));
            file.store(&path_to_store)?;
            POPUPS_MANAGER
                .lock()
                .create_added_resource(&file.resource)?;
            return Ok(());
        }

        let path = uri.trim_start_matches(Self::URI).trim_start_matches("/");
        let parts = path.split("/").map(|s| s.to_string()).collect_vec();

        if parts.len() != 3 {
            return Err("Invalid URI format".into());
        }

        let [_method, enviroment, resource_id] = parts.as_slice() else {
            return Err("Invalid URI format".into());
        };
        let Ok(resource_id) = Uuid::parse_str(resource_id) else {
            return Err("Invalid URI format".into());
        };

        let env_prefix = if enviroment == "production" {
            "".to_string()
        } else {
            format!(".{enviroment}")
        };

        let url = format!("https://product{env_prefix}.seelen.io/resource/download/{resource_id}");
        tauri::async_runtime::block_on(async move {
            let res = reqwest::get(url).await?;
            let file = res.json::<SluResourceFile>().await?;
            let path_to_store = path_by_resource_kind(&file.resource.kind)
                .join(format!("{}.slu", file.resource.id));
            file.store(&path_to_store)?;
            POPUPS_MANAGER
                .lock()
                .create_added_resource(&file.resource)?;
            Result::Ok(())
        })?;
        Ok(())
    }

    pub fn process(self) -> Result<()> {
        if let Some(uri) = self.uri {
            return Self::process_uri(&uri);
        }

        let Some(command) = self.command else {
            return Ok(());
        };

        match command {
            AppCliCommand::Settings => {
                Seelen::show_settings()?;
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
        }
        Ok(())
    }

    /// will fail if no instance is running
    pub fn send_to_main_instance(self) -> Result<()> {
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

        let stream = SelfPipe::connect_tcp()?;
        serde_json::to_writer(stream, &args)?;
        Ok(())
    }
}
