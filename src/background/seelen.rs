use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, Wry};
use tauri_plugin_shell::ShellExt;

use crate::{error_handler::Result, hook::register_hook, seelenweg::SeelenWeg, state::State};

lazy_static! {
    pub static ref SEELEN: Arc<Mutex<Seelen>> = Arc::new(Mutex::new(Seelen::default()));
}

pub struct Seelen {
    handle: Option<AppHandle<Wry>>,
    weg: Option<SeelenWeg>,
    state: State,
}

impl Default for Seelen {
    fn default() -> Self {
        Self {
            handle: None,
            weg: None,
            state: State::default(),
        }
    }
}

impl Seelen {
    pub fn handle(&self) -> &AppHandle<Wry> {
        self.handle.as_ref().unwrap()
    }

    pub fn weg_mut(&mut self) -> &mut SeelenWeg {
        self.weg.as_mut().unwrap()
    }

    pub fn weg(&self) -> &SeelenWeg {
        self.weg.as_ref().unwrap()
    }
    pub fn init(&mut self, app: AppHandle<Wry>) {
        log::trace!("Initializing Seelen");
        self.handle = Some(app.clone());
        self.weg = Some(SeelenWeg::new(app.clone()));
        self.state = State::new(
            &app.path()
                .resolve(".config/komorebi-ui/settings.json", BaseDirectory::Home)
                .expect("Failed to resolve path"),
        )
        .ok()
        .unwrap_or_default();
    }

    pub fn start(&mut self) {
        self.ensure_folders().expect("Fail on ensuring folders");
        self.start_ahk_shortcuts();
        self.start_komorebi_manager();

        if self.state.is_weg_enabled() {
            match self.weg_mut().start() {
                Ok(_) => {
                    register_hook().expect("Failed to register hook");
                }
                Err(err) => log::error!("Fail on starting SeelenWeg: {err}"),
            };
        }
    }

    pub fn stop(&self) {
        self.kill_ahk_shortcuts();

        if self.state.is_weg_enabled() {
            self.weg().stop();
        }
    }

    pub fn ensure_folders(&self) -> Result<()> {
        log::trace!("Ensuring folders");
        let path = self.handle().path();

        // komorebi window manager does not create this folder on first install/run 🤡
        std::fs::create_dir_all(path.resolve("komorebi", BaseDirectory::LocalData)?)?;
        std::fs::create_dir_all(path.resolve(".config/komorebi-ui", BaseDirectory::Home)?)?;
        std::fs::create_dir_all(path.resolve("gen", BaseDirectory::Resource)?)?;

        Ok(())
    }

    pub fn start_ahk_shortcuts(&self) {
        log::trace!("Starting AHK shortcuts");

        tauri::async_runtime::spawn(async move {
            let app = SEELEN.lock().handle().clone();

            let ahk_path = app
                .path()
                .resolve("static/seelen.ahk", BaseDirectory::Resource)
                .expect("Failed to resolve path")
                .to_str()
                .expect("Failed to convert path to string")
                .to_owned()
                .trim_start_matches("\\\\?\\")
                .to_owned();

            app.shell()
                .command("cmd")
                .args(["/C", &ahk_path])
                .spawn()
                .expect("Failed to spawn shortcuts");
        });
    }

    pub fn kill_ahk_shortcuts(&self) {
        log::trace!("Killing AHK shortcuts");

        self.handle()
            .shell()
            .command("powershell")
            .args([
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-Command",
                "Get-WmiObject Win32_Process | Where-Object { $_.CommandLine -like '*seelen.ahk*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }",
            ])
            .spawn()
            .expect("Failed to close ahk");
    }

    pub fn start_komorebi_manager(&self) {
        log::trace!("Starting komorebi manager");

        tauri::async_runtime::spawn(async move {
            let app = SEELEN.lock().handle().clone();

            let config_route = app
                .path()
                .resolve(".config/komorebi-ui/settings.json", BaseDirectory::Home)
                .expect("Failed to resolve path")
                .to_str()
                .unwrap_or("")
                .to_string();

            app.shell()
                .command("komorebi-wm.exe")
                .args(["-c", &config_route])
                .spawn()
                .expect("Failed to spawn komorebi");
        });
    }
}
