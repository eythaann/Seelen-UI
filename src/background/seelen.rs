use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;

use crate::{
    apps_config::SETTINGS_BY_APP, error_handler::{log_if_error, Result}, hook::register_hook_and_enum_windows, k_killer::WindowManager, seelen_bar::SeelenBar, seelen_shell::SeelenShell, seelenweg::SeelenWeg, state::State, utils::run_ahk_file
};

lazy_static! {
    pub static ref SEELEN: Arc<Mutex<Seelen>> = Arc::new(Mutex::new(Seelen::default()));
}

/** Struct should be initialized first before calling any other methods */
pub struct Seelen {
    handle: Option<AppHandle<Wry>>,
    weg: Option<SeelenWeg>,
    bar: Option<SeelenBar>,
    shell: Option<SeelenShell>,
    window_manager: Option<WindowManager>,
    state: State,
    pub initialized: bool,
}

impl Default for Seelen {
    fn default() -> Self {
        Self {
            handle: None,
            weg: None,
            bar: None,
            shell: None,
            window_manager: None,
            state: State::default(),
            initialized: false,
        }
    }
}

/* ============== Getters ============== */
impl Seelen {
    /** Ensure Seelen is initialized first before calling */
    pub fn handle(&self) -> &AppHandle<Wry> {
        self.handle.as_ref().unwrap()
    }

    pub fn weg_mut(&mut self) -> Option<&mut SeelenWeg> {
        self.weg.as_mut()
    }

    pub fn weg(&self) -> Option<&SeelenWeg> {
        self.weg.as_ref()
    }

    /* pub fn bar(&self) -> Option<&SeelenBar> {
        self.bar.as_ref()
    }

    pub fn bar_mut(&mut self) -> Option<&mut SeelenBar> {
        self.bar.as_mut()
    }

    pub fn shell(&self) -> Option<&SeelenShell> {
        self.shell.as_ref()
    }

    pub fn shell_mut(&mut self) -> Option<&mut SeelenShell> {
        self.shell.as_mut()
    } */

    pub fn wm_mut(&mut self) -> Option<&mut WindowManager> {
        self.window_manager.as_mut()
    }

    pub fn wm(&self) -> Option<&WindowManager> {
        self.window_manager.as_ref()
    }
}

/* ============== Methods ============== */
impl Seelen {
    pub fn init(&mut self, app: AppHandle<Wry>) -> Result<()> {
        log::trace!("Initializing Seelen");
        self.handle = Some(app.clone());

        let mut path = app.path().resolve(".config/seelen/settings.json", BaseDirectory::Home)?;
        if !path.exists() {
            path = app.path().resolve(".config/komorebi-ui/settings.json", BaseDirectory::Home)?;
        }
        self.state = State::new(&path).unwrap_or_default();

        let mut path = app.path().resolve(".config/seelen/applications.yml", BaseDirectory::Home)?;
        if !path.exists() {
            path = app.path().resolve(".config/komorebi-ui/applications.yml", BaseDirectory::Home)?;
        }
        SETTINGS_BY_APP.lock().load(path);

        if self.state.is_weg_enabled() {
            self.weg = Some(SeelenWeg::new(app.clone()));
        }

        if self.state.is_shell_enabled() {
            self.shell = Some(SeelenShell::new(app.clone()));
        }

        if self.state.is_bar_enabled() {
            self.bar = Some(SeelenBar::new(app.clone()));
        }

        if self.state.is_window_manager_enabled() {
            self.window_manager = Some(WindowManager::new(app.clone()));
        }

        self.initialized = true;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        self.ensure_folders().expect("Fail on ensuring folders");
        self.start_ahk_shortcuts()?;

        if let Some(weg) = self.weg_mut() {
            log_if_error(weg.start());
        }

        register_hook_and_enum_windows()?;
        Ok(())
    }

    pub fn stop(&self) {
        if self.state.is_ahk_enabled() {
            self.kill_ahk_shortcuts();
        }

        if let Some(weg) = self.weg() {
            weg.stop();
        }
    }

    pub fn ensure_folders(&self) -> Result<()> {
        log::trace!("Ensuring folders");
        let path = self.handle().path();
        std::fs::create_dir_all(path.resolve(".config/seelen", BaseDirectory::Home)?)?;
        Ok(())
    }

    pub fn start_ahk_shortcuts(&self) -> Result<()> {
        if self.state.is_ahk_enabled() {
            run_ahk_file(self.handle(), "seelen.ahk")?;
            if self.wm().is_some(){
                run_ahk_file(self.handle(), "seelen.wm.ahk")?;
            }
        }
        Ok(())
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

    pub fn show_settings(&self) -> Result<WebviewWindow> {
        log::trace!("show_settings_window");
        let window = tauri::WebviewWindowBuilder::new(
            self.handle(),
            "settings",
            tauri::WebviewUrl::App("settings/index.html".into()),
        )
        .inner_size(700.0, 500.0)
        .maximizable(false)
        .minimizable(true)
        .resizable(false)
        .title("Settings")
        .visible(false)
        .decorations(false)
        .center()
        .build()?;
        Ok(window)
    }

    pub fn create_update_modal(&self) -> Result<()> {
        log::trace!("Creating update notification window");

        // check if path is in windowsapps folder
        let installation_path = self.handle().path().resource_dir()?;
        if installation_path
            .to_string_lossy()
            .contains(r"\Program Files\WindowsApps\")
        {
            log::trace!("Skipping update notification because it is installed as MSIX");
            return Ok(());
        }

        tauri::WebviewWindowBuilder::new(
            self.handle(),
            "updater",
            tauri::WebviewUrl::App("update/index.html".into()),
        )
        .inner_size(500.0, 240.0)
        .maximizable(false)
        .minimizable(true)
        .resizable(false)
        .title("Update Available")
        .visible(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .center()
        .always_on_top(true)
        .build()?;

        Ok(())
    }
}
