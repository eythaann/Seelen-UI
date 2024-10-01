// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error_handler;
mod exposed;
mod hook;
mod instance;
mod modules;
mod plugins;
mod seelen;
mod seelen_bar;
mod seelen_rofi;
mod seelen_wall;
mod seelen_weg;
mod seelen_wm_v2;
mod state;
mod system;
mod tray;
mod utils;
mod windows_api;
mod winevent;

use std::io::{BufWriter, Write};

use color_eyre::owo_colors::OwoColorize;
use error_handler::Result;
use exposed::register_invoke_handler;
use itertools::Itertools;
use modules::{
    cli::{
        application::{attach_console, is_just_getting_info, SEELEN_COMMAND_LINE},
        Client,
    },
    tray::application::ensure_tray_overflow_creation,
    uwp::UWP_MANAGER,
};
use plugins::register_plugins;
use seelen::{Seelen, SEELEN};
use seelen_core::state::Settings;
use tauri::webview_version;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_shell::ShellExt;
use tray::try_register_tray_icon;
use utils::{spawn_named_thread, PERFORMANCE_HELPER};
use windows::Win32::Security::{SE_DEBUG_NAME, SE_SHUTDOWN_NAME};
use windows_api::WindowsApi;

fn register_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let cause = info
            .payload()
            .downcast_ref::<String>()
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                info.payload()
                    .downcast_ref::<&str>()
                    .unwrap_or(&"<cause unknown>")
                    .to_string()
            });

        let mut string_location = String::from("<location unknown>");
        if let Some(location) = info.location() {
            string_location = format!(
                "{}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }

        log::error!(
            "A panic occurred:\n  Cause: {}\n  Location: {}",
            cause.cyan(),
            string_location.purple()
        );
    }));
}

fn print_initial_information() {
    let version = env!("CARGO_PKG_VERSION");
    log::info!("───────────────────── Starting Seelen UI v{version} ─────────────────────");
    log::info!("Operating System: {}", os_info::get());
    log::info!("WebView2 Runtime: {:?}", webview_version());
    log::info!("Elevated        : {:?}", WindowsApi::is_elevated());
    log::info!("Locate          : {:?}", Settings::get_locale());
}

fn setup(app: &mut tauri::App<tauri::Wry>) -> Result<(), Box<dyn std::error::Error>> {
    print_initial_information();
    Client::listen_tcp()?;

    if webview_version().is_err() {
        let ok_pressed = app
            .dialog()
            .message("Seelen UI requires Webview2 Runtime. Please install it.")
            .title("WebView2 Runtime not found")
            .kind(MessageDialogKind::Error)
            .ok_button_label("Go to download page")
            .blocking_show();

        if ok_pressed {
            app.shell().open(
            "https://developer.microsoft.com/en-us/microsoft-edge/webview2/?form=MA13LH#download",
            None,
        )?;
        }
        app.handle().exit(1);
        return Ok(());
    }

    let mut seelen = trace_lock!(SEELEN);
    seelen.init(app.handle())?;

    log_error!(WindowsApi::enable_privilege(SE_SHUTDOWN_NAME));
    log_error!(WindowsApi::enable_privilege(SE_DEBUG_NAME));

    // init the UWP manager this took a long time so we spawn it in the background
    // the most of the apps need this so we init it and the beginning of the program
    spawn_named_thread("UWP Manager Init", || {
        // lazy variables are initialized when they are accessed
        UWP_MANAGER.is_locked();
    })?;

    // try it at start it on open the program to avoid do it before
    log_error!(ensure_tray_overflow_creation());

    if !tauri::is_dev() {
        let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
        if !command.get_matches().get_flag("silent") {
            Seelen::show_settings()?;
        }
    }

    seelen.start()?;
    log_error!(try_register_tray_icon(app));
    trace_lock!(PERFORMANCE_HELPER).end("setup");
    Ok(())
}

fn app_callback(_: &tauri::AppHandle<tauri::Wry>, event: tauri::RunEvent) {
    match event {
        tauri::RunEvent::ExitRequested { api, code, .. } => {
            // prevent close background on webview windows closing
            if code.is_none() {
                api.prevent_exit();
            }
        }
        tauri::RunEvent::Exit => {
            log::info!("───────────────────── Exiting Seelen UI ─────────────────────");
            if Seelen::is_running() {
                trace_lock!(SEELEN).stop();
            }
        }
        _ => {}
    }
}

fn is_already_runnning() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes();
    sys.processes()
        .values()
        .filter(|p| p.exe().is_some_and(|path| path.ends_with("seelen-ui.exe")))
        .collect_vec()
        .len()
        > 1
}

fn main() -> Result<()> {
    color_eyre::install().expect("Failed to install color_eyre");
    register_panic_hook();
    trace_lock!(PERFORMANCE_HELPER).start("setup");

    let command = trace_lock!(SEELEN_COMMAND_LINE).clone();
    let matches = match command.try_get_matches() {
        Ok(m) => m,
        // (help, --help or -h) are managed as error
        Err(e) => {
            attach_console()?;
            e.print()?;
            return Ok(());
        }
    };

    if is_just_getting_info(&matches)? {
        return Ok(());
    }

    if is_already_runnning() {
        let mut attempts = 0;
        let mut connection = Client::connect_tcp();

        while connection.is_err() && attempts < 10 {
            attempts += 1;
            std::thread::sleep(std::time::Duration::from_millis(100));
            connection = Client::connect_tcp();
        }

        if let Ok(stream) = connection {
            let mut writer = BufWriter::new(stream);

            let args = std::env::args().collect_vec();
            let msg = serde_json::to_string(&args).expect("could not serialize");

            writer.write_all(msg.as_bytes()).expect("could not write");
            writer.flush().expect("could not flush");
            return Ok(());
        }

        // if the connection fails probably is because the app is been closing
        // so we check if the app is already running again to see if we have to
        // let the instance be created or not
        if is_already_runnning() {
            return Ok(());
        }
    }

    let mut app_builder = tauri::Builder::default();
    app_builder = register_plugins(app_builder);
    app_builder = register_invoke_handler(app_builder);

    let app = app_builder
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(app_callback);
    Ok(())
}
