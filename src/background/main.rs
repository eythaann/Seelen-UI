// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod error_handler;
mod tray;
mod windows;

use std::sync::Arc;

use cli::handle_cli;
use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, Wry};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_shell::ShellExt;
use tray::handle_tray_icon;
use windows::set_windows_events;

pub struct Seelen {
    handle: Option<AppHandle<Wry>>,
}

impl Default for Seelen {
    fn default() -> Self {
        Self { handle: None }
    }
}

impl Seelen {
    pub fn handle(&self) -> &AppHandle<Wry> {
        self.handle.as_ref().unwrap()
    }

    pub fn set_handle(&mut self, app: AppHandle<Wry>) {
        self.handle = Some(app);
    }
}

lazy_static! {
    pub static ref SEELEN: Arc<Mutex<Seelen>> = Arc::new(Mutex::new(Seelen::default()));
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--silent"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            app.emit("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .setup(|app| {
            SEELEN.lock().set_handle(app.handle().clone());
            set_windows_events(app)?;

            let config_route = app
                .path()
                .resolve(".config/komorebi-ui/settings.json", BaseDirectory::Home)?
                .to_str()
                .unwrap_or("")
                .to_string();

            tauri::async_runtime::spawn(async move {
                let app = SEELEN.lock().handle().clone();

                app.shell()
                    .command("komorebi.exe")
                    .args(["-c", &config_route])
                    .spawn()
                    .expect("Failed to spawn komorebi");

                app.shell()
                    .command("cmd")
                    .args(["/C", ".\\static\\shortcuts.ahk"])
                    .spawn()
                    .expect("Failed to spawn shortcuts");
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    let should_run_app = handle_cli(&mut app)?;
    if !should_run_app {
        return Ok(());
    }

    handle_tray_icon(&mut app)?;

    app.run(|_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        _ => {}
    });

    Ok(())
}
