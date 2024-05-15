use std::{env::temp_dir, sync::Arc};

use getset::{Getters, MutGetters};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{HDC, HMONITOR},
    UI::WindowsAndMessaging::EnumWindows,
};

use crate::{
    apps_config::SETTINGS_BY_APP,
    error_handler::{log_if_error, Result},
    hook::register_win_hook,
    monitor::Monitor,
    seelen_shell::SeelenShell,
    seelen_weg::SeelenWeg,
    seelen_wm::WindowManager,
    state::State,
    system::register_system_events,
    utils::{run_ahk_file, sleep_millis},
    windows_api::WindowsApi,
};

lazy_static! {
    pub static ref SEELEN: Arc<Mutex<Seelen>> = Arc::new(Mutex::new(Seelen::default()));
    pub static ref APP_HANDLE: Arc<Mutex<Option<AppHandle<Wry>>>> = Arc::new(Mutex::new(None));
    pub static ref APP_STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::default()));
}

pub fn get_app_handle() -> AppHandle<Wry> {
    APP_HANDLE
        .lock()
        .clone()
        .expect("get_app_handle called but app is still not initialized")
}

/** Struct should be initialized first before calling any other methods */
#[derive(Getters, MutGetters)]
pub struct Seelen {
    handle: Option<AppHandle<Wry>>,
    #[getset(get = "pub", get_mut = "pub")]
    monitors: Vec<Monitor>,
    shell: Option<SeelenShell>,
    #[getset(get = "pub")]
    state: State,
    pub initialized: bool,
}

impl Default for Seelen {
    fn default() -> Self {
        Self {
            handle: None,
            monitors: Vec::new(),
            shell: None,
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

    pub fn focused_monitor(&self) -> Option<&Monitor> {
        self.monitors.iter().find(|m| m.is_focused())
    }

    pub fn focused_monitor_mut(&mut self) -> Option<&mut Monitor> {
        self.monitors.iter_mut().find(|m| m.is_focused())
    }

    pub fn monitor_by_id_mut(&mut self, id: isize) -> Option<&mut Monitor> {
        self.monitors.iter_mut().find(|m| m.hmonitor().0 == id)
    }

    /* pub fn shell(&self) -> Option<&SeelenShell> {
        self.shell.as_ref()
    }

    pub fn shell_mut(&mut self) -> Option<&mut SeelenShell> {
        self.shell.as_mut()
    } */
}

/* ============== Methods ============== */
impl Seelen {
    pub fn init(&mut self, app: AppHandle<Wry>) -> Result<()> {
        log::trace!("Initializing Seelen");
        self.handle = Some(app.clone());
        *APP_HANDLE.lock() = Some(app.clone());

        self.ensure_folders()?;
        self.load_uwp_apps_info()?;

        let path = app
            .path()
            .resolve(".config\\seelen\\settings.json", BaseDirectory::Home)?;
        self.state = State::new(&path).unwrap_or_default();
        *APP_STATE.lock() = self.state.clone();

        let mut settings_by_app = SETTINGS_BY_APP.lock();
        settings_by_app.set_paths(
            app.path()
                .resolve(".config\\seelen\\applications.yml", BaseDirectory::Home)?,
            app.path()
                .resolve("static\\apps_templates", BaseDirectory::Resource)?,
        );
        log_if_error(settings_by_app.load());

        if self.state.is_shell_enabled() {
            self.shell = Some(SeelenShell::new(app.clone()));
        }

        self.initialized = true;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        if self.state.is_weg_enabled() {
            SeelenWeg::hide_taskbar(true);
        }

        self.start_ahk_shortcuts()?;

        Self::enum_monitors();
        register_win_hook()?;
        register_system_events(self.handle().clone())?;
        Ok(())
    }

    pub fn stop(&self) {
        if self.state.is_weg_enabled() {
            SeelenWeg::hide_taskbar(false);
        }
        if self.state.is_ahk_enabled() {
            self.kill_ahk_shortcuts();
        }
    }

    pub fn ensure_folders(&self) -> Result<()> {
        log::trace!("Ensuring folders");
        let path = self.handle().path();
        std::fs::create_dir_all(path.resolve(".config/seelen", BaseDirectory::Home)?)?;
        Ok(())
    }

    pub fn load_uwp_apps_info(&self) -> Result<()> {
        let pwsh_script = include_str!("load_uwp_apps.ps1");
        let pwsh_script_path = temp_dir().join("load_uwp_apps.ps1");
        std::fs::write(&pwsh_script_path, pwsh_script).expect("Failed to write temp script file");

        tauri::async_runtime::block_on(async move {
            let result = self
                .handle()
                .shell()
                .command("powershell")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-NoProfile",
                    "-File",
                    &pwsh_script_path.to_string_lossy(),
                    "-SavePath",
                    &self
                        .handle()
                        .path()
                        .app_data_dir()
                        .expect("Failed to resolve app data dir")
                        .join("uwp_manifests.json")
                        .to_string_lossy()
                        .trim_start_matches("\\\\?\\"),
                ])
                .status()
                .await;

            match result {
                Ok(status) => {
                    log::trace!(
                        "load_uwp_apps exit code: {}",
                        status.code().unwrap_or_default()
                    );
                }
                Err(err) => log::error!("load_uwp_apps Failed to wait for process: {}", err),
            };
        });
        Ok(())
    }

    pub fn start_ahk_shortcuts(&self) -> Result<()> {
        if self.state.is_ahk_enabled() {
            run_ahk_file(self.handle(), "seelen.ahk")?;

            if self.state.is_window_manager_enabled() {
                log_if_error(run_ahk_file(self.handle(), "seelen.wm.ahk"));
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
                r"Get-WmiObject Win32_Process | Where-Object { $_.CommandLine -like '*static\redis\AutoHotkey.exe*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }",
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

        // check if path is in windows apps folder
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

impl Seelen {
    unsafe extern "system" fn enum_monitors_proc(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _rect_clip: *mut RECT,
        _lparam: LPARAM,
    ) -> BOOL {
        let mut seelen = SEELEN.lock();
        match Monitor::new(hmonitor, &seelen.state) {
            Ok(monitor) => seelen.monitors.push(monitor),
            Err(err) => log::error!("Failed to create monitor: {:?}", err),
        }
        true.into()
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut seelen = SEELEN.lock();
        for monitor in seelen.monitors_mut() {
            if let Some(weg) = monitor.weg_mut() {
                if SeelenWeg::is_real_window(hwnd, false) {
                    weg.add_hwnd(hwnd);
                }
            }

            if let Some(wm) = monitor.wm_mut() {
                if WindowManager::is_manageable_window(hwnd, true) {
                    log_if_error(wm.add_hwnd(hwnd));
                }
            }
        }
        true.into()
    }

    pub fn enum_monitors() {
        std::thread::spawn(|| unsafe {
            log::trace!("Enumerating Monitors");
            log_if_error(WindowsApi::enum_display_monitors(
                Some(Self::enum_monitors_proc),
                0,
            ));
            log::trace!("Finished enumerating Monitors");

            let mut all_ready = false;
            while !all_ready {
                all_ready = SEELEN.lock().monitors().iter().all(|m| m.is_ready());
                sleep_millis(10);
            }

            log::trace!("Enumerating windows");
            log_if_error(EnumWindows(Some(Self::enum_windows_proc), LPARAM(0)));
            log::trace!("Finished enumerating windows");
        });
    }
}
