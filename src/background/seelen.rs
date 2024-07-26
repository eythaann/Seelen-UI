use std::{env::temp_dir, sync::Arc};

use getset::{Getters, MutGetters};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewWindow, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{HDC, HMONITOR},
};

use crate::{
    apps_config::SETTINGS_BY_APP,
    error_handler::Result,
    hook::register_win_hook,
    log_error,
    monitor::Monitor,
    seelen_shell::SeelenShell,
    seelen_weg::SeelenWeg,
    seelen_wm::WindowManager,
    state::State,
    system::{declare_system_events_handlers, release_system_events_handlers},
    trace_lock,
    utils::{ahk::AutoHotKey, sleep_millis},
    windows_api::WindowsApi,
};

lazy_static! {
    pub static ref SEELEN: Arc<Mutex<Seelen>> = Arc::new(Mutex::new(Seelen::default()));
    pub static ref APP_HANDLE: Arc<Mutex<Option<AppHandle<Wry>>>> = Arc::new(Mutex::new(None));
}

pub fn get_app_handle() -> AppHandle<Wry> {
    APP_HANDLE
        .lock()
        .clone()
        .expect("get_app_handle called but app is still not initialized")
}

/** Struct should be initialized first before calling any other methods */
#[derive(Getters, MutGetters, Default)]
pub struct Seelen {
    handle: Option<AppHandle<Wry>>,
    #[getset(get = "pub", get_mut = "pub")]
    monitors: Vec<Monitor>,
    shell: Option<SeelenShell>,
    #[getset(get = "pub")]
    state: State,
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
    pub fn refresh_state(&mut self) -> Result<()> {
        self.state.refresh()
    }

    pub fn init(&mut self, app: AppHandle<Wry>) -> Result<()> {
        log::trace!("Initializing Seelen");
        {
            self.handle = Some(app.clone());
            *APP_HANDLE.lock() = Some(app.clone());
        }

        self.ensure_folders()?;
        self.load_uwp_apps_info()?;

        let data_path = app.path().app_data_dir()?;
        self.state = State::new(&data_path.join("settings.json")).unwrap_or_default();

        let mut settings_by_app = trace_lock!(SETTINGS_BY_APP);
        settings_by_app.set_paths(
            data_path.join("applications.yml"),
            app.path()
                .resolve("static\\apps_templates", BaseDirectory::Resource)?,
        );
        log_error!(settings_by_app.load());

        if self.state.is_shell_enabled() {
            self.shell = Some(SeelenShell::new(app.clone()));
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        if self.state.is_weg_enabled() {
            //SeelenWeg::hide_taskbar(true)?;
        }

        self.start_ahk_shortcuts()?;
        declare_system_events_handlers()?;

        std::thread::spawn(|| {
            log::trace!("Enumerating Monitors");
            WindowsApi::enum_display_monitors(Some(Self::enum_monitors_proc), 0)
                .expect("Failed to enum monitors");

            let mut all_ready = false;
            while !all_ready {
                sleep_millis(10);
                all_ready = trace_lock!(SEELEN).monitors().iter().all(|m| m.is_ready());
            }

            log::trace!("Enumerating windows");
            WindowsApi::enum_windows(Some(Self::enum_windows_proc), 0)
                .expect("Failed to enum windows");

            register_win_hook().expect("Failed to register windows hook");
        });

        if WindowsApi::is_elevated()? {
            Self::bind_file_extensions()?;

            if Self::is_auto_start_enabled()? {
                // override auto-start task in case of location change, normally this happen on MSIX update
                self.set_auto_start(true)?;
            }
        }

        Ok(())
    }

    /// Stop and release all resources
    pub fn stop(&self) {
        release_system_events_handlers();
        if self.state.is_weg_enabled() {
            log_error!(SeelenWeg::hide_taskbar(false));
        }
        if self.state.is_ahk_enabled() {
            self.kill_ahk_shortcuts();
        }
    }

    fn bind_file_extensions() -> Result<()> {
        use crate::modules::file_extensions::infrastructure::*;

        Theme::create_uri_protocol()?;
        Theme::create_ext_protocol()?;

        Ok(())
    }

    fn ensure_folders(&self) -> Result<()> {
        log::trace!("Ensuring folders");
        let path = self.handle().path();
        let data_path = path.app_data_dir()?;

        // migration of user settings files below v1.8.3
        let old_path = path.resolve(".config/seelen", BaseDirectory::Home)?;
        if old_path.exists() {
            log::trace!("Migrating user settings from {:?}", old_path);
            for entry in std::fs::read_dir(&old_path)?.flatten() {
                if entry.file_type()?.is_dir() {
                    continue;
                }
                std::fs::copy(entry.path(), data_path.join(entry.file_name()))?;
            }
            std::fs::remove_dir_all(&old_path)?;
        }

        // user data folder
        std::fs::create_dir_all(data_path.join("placeholders"))?;
        std::fs::create_dir_all(data_path.join("themes"))?;
        std::fs::create_dir_all(data_path.join("layouts"))?;
        std::fs::create_dir_all(data_path.join("icons"))?;

        Ok(())
    }

    pub fn is_auto_start_enabled() -> Result<bool> {
        let handle = get_app_handle();
        let output = tauri::async_runtime::block_on(async move {
            handle
                .shell()
                .command("powershell")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-NoProfile",
                    "-Command",
                    "[bool](Get-ScheduledTask -TaskName Seelen-UI -ErrorAction SilentlyContinue)",
                ])
                .output()
                .await
        })?;
        let stdout = String::from_utf8(output.stdout)?.trim().to_lowercase();
        Ok(stdout == "true")
    }

    pub fn set_auto_start(&self, enabled: bool) -> Result<()> {
        let pwsh_script = include_str!("schedule.ps1");
        let pwsh_script_path = temp_dir().join("schedule.ps1");
        std::fs::write(&pwsh_script_path, pwsh_script).expect("Failed to write temp script file");

        let exe_path = std::env::current_exe()?;

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
                    "-ExeRoute",
                    &exe_path.to_string_lossy(),
                    "-Enabled",
                    if enabled { "true" } else { "false" },
                ])
                .output()
                .await;

            match result {
                Ok(output) => {
                    if !output.status.success() {
                        let stderr = std::str::from_utf8(&output.stderr)
                            .unwrap_or_default()
                            .trim();

                        log::error!("schedule auto start failed: {}", stderr);
                    }
                }
                Err(err) => log::error!("schedule auto start Failed to wait for process: {}", err),
            };
        });

        Ok(())
    }

    fn load_uwp_apps_info(&self) -> Result<()> {
        let time = std::time::Instant::now();
        let pwsh_script = include_str!("load_uwp_apps.ps1");
        let pwsh_script_path = temp_dir().join("load_uwp_apps.ps1");
        std::fs::write(&pwsh_script_path, pwsh_script).expect("Failed to write temp script file");

        let manifest_path = self
            .handle()
            .path()
            .app_data_dir()?
            .join("uwp_manifests.json");
        let manifest_path_exist = manifest_path.exists();

        let task = async move {
            let result = get_app_handle()
                .shell()
                .command("powershell")
                .args([
                    "-ExecutionPolicy",
                    "Bypass",
                    "-NoProfile",
                    "-File",
                    &pwsh_script_path.to_string_lossy(),
                    "-SavePath",
                    manifest_path
                        .to_string_lossy()
                        .trim_start_matches("\\\\?\\"),
                ])
                .status()
                .await;

            let duration = time.elapsed();
            match result {
                Ok(status) => log::trace!(
                    "load_uwp_apps took: {}s, exit code: {}",
                    duration.as_secs_f32(),
                    status.code().unwrap_or_default()
                ),
                Err(err) => log::error!(
                    "load_uwp_apps took: {}, failed to wait for process: {}",
                    duration.as_secs_f32(),
                    err
                ),
            };
        };

        if !manifest_path_exist {
            tauri::async_runtime::block_on(task);
        } else {
            tauri::async_runtime::spawn(task);
        }

        Ok(())
    }

    pub fn start_ahk_shortcuts(&self) -> Result<()> {
        // kill all running shortcuts before starting again
        self.kill_ahk_shortcuts();

        if self.state.is_ahk_enabled() {
            log::trace!("Starting seelen.ahk");
            AutoHotKey::new(include_str!("utils/ahk/mocks/seelen.ahk"))
                .with_lib()
                .execute()?;

            if self.state.is_window_manager_enabled() {
                log::trace!("Starting seelen.wm.ahk");
                AutoHotKey::from_template(
                    include_str!("utils/ahk/mocks/seelen.wm.ahk"),
                    self.state.get_ahk_variables(),
                )
                .with_lib()
                .execute()?;
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
        log::trace!("Show settings window");

        let window = self.handle().get_webview_window("settings").or_else(|| {
            tauri::WebviewWindowBuilder::new(
                self.handle(),
                "settings",
                tauri::WebviewUrl::App("settings/index.html".into()),
            )
            .title("Settings")
            .inner_size(720.0, 480.0)
            .maximizable(false)
            .minimizable(true)
            .resizable(false)
            .visible(false)
            .decorations(false)
            .center()
            .build()
            .ok()
        });

        match window {
            Some(window) => {
                window.unminimize()?;
                window.set_focus()?;
                Ok(window)
            }
            None => Err("Failed to create settings window".into()),
        }
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
        let mut seelen = trace_lock!(SEELEN);
        match Monitor::new(hmonitor, &seelen.state) {
            Ok(monitor) => seelen.monitors.push(monitor),
            Err(err) => log::error!("Failed to create monitor: {:?}", err),
        }
        true.into()
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut seelen = trace_lock!(SEELEN);
        for monitor in seelen.monitors_mut() {
            if let Some(weg) = monitor.weg_mut() {
                if SeelenWeg::is_real_window(hwnd, false) {
                    weg.add_hwnd(hwnd);
                }
            }

            if let Some(wm) = monitor.wm_mut() {
                if WindowManager::is_manageable_window(hwnd, true) {
                    log_error!(wm.add_hwnd(hwnd));
                }
            }
        }
        true.into()
    }
}
