use std::{env::temp_dir, sync::Arc};

use arc_swap::ArcSwap;
use getset::{Getters, MutGetters};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    Graphics::Gdi::HMONITOR,
};

use crate::{
    error_handler::Result,
    hook::register_win_hook,
    log_error,
    modules::monitors::{MonitorManagerEvent, MONITOR_MANAGER},
    monitor::Monitor,
    seelen_shell::SeelenShell,
    seelen_weg::SeelenWeg,
    seelen_wm::WindowManager,
    state::application::{FullState, FULL_STATE},
    system::{declare_system_events_handlers, release_system_events_handlers},
    trace_lock,
    utils::{ahk::AutoHotKey, sleep_millis, spawn_named_thread, PERFORMANCE_HELPER},
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
    state: Option<Arc<ArcSwap<FullState>>>,
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
        self.monitors.iter_mut().find(|m| m.handle().0 == id)
    }

    pub fn monitor_by_name_mut(&mut self, name: &str) -> Option<&mut Monitor> {
        self.monitors.iter_mut().find(|m| m.name() == name)
    }

    pub fn state(&self) -> Arc<FullState> {
        self.state
            .as_ref()
            .expect("Seelen State not initialized")
            .load_full()
    }
}

/* ============== Methods ============== */
impl Seelen {
    pub fn on_state_changed(&mut self) -> Result<()> {
        let state = self.state();
        for monitor in &mut self.monitors {
            monitor.load_settings(&state)?;
        }
        Ok(())
    }

    pub fn init(&mut self, app: AppHandle<Wry>) -> Result<()> {
        Self::ensure_folders(&app)?;

        log::trace!("Initializing Seelen");
        {
            *APP_HANDLE.lock() = Some(app.clone());
            self.handle = Some(app.clone());
            self.state = Some(Arc::clone(&FULL_STATE));
        }

        if self.state().is_shell_enabled() {
            self.shell = Some(SeelenShell::new(app.clone()));
        }

        Ok(())
    }

    fn on_monitor_event(event: MonitorManagerEvent) {
        log::trace!("Monitor event: {:?}", event);
        let mut seelen = trace_lock!(SEELEN);
        match event {
            MonitorManagerEvent::Added(_name, id) => {
                log_error!(seelen.add_monitor(id));
            }
            MonitorManagerEvent::Removed(_name, id) => {
                log_error!(seelen.remove_monitor(id));
            }
            MonitorManagerEvent::Updated(name, id) => {
                if let Some(m) = seelen.monitor_by_name_mut(&name) {
                    m.update_handle(id);
                }
            }
        }
    }

    fn start_async() -> Result<()> {
        log_error!(Self::start_ahk_shortcuts());

        let mut all_ready = false;
        while !all_ready {
            sleep_millis(50);
            all_ready = SEELEN.lock().monitors().iter().all(|m| m.is_ready());
        }

        log::debug!(
            "Seelen UI ready in: {:.2}s",
            PERFORMANCE_HELPER.lock().elapsed().as_secs_f64()
        );

        log::trace!("Enumerating windows");
        WindowsApi::enum_windows(Some(Self::enum_windows_proc), 0)?;
        register_win_hook()?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        declare_system_events_handlers()?;

        if self.state().is_weg_enabled() {
            SeelenWeg::hide_taskbar();
        }

        log::trace!("Enumerating Monitors");
        let mut monitor_manager = trace_lock!(MONITOR_MANAGER);
        for (_name, id) in &monitor_manager.monitors {
            log_error!(self.add_monitor(*id));
        }
        monitor_manager.listen_changes(Self::on_monitor_event);

        spawn_named_thread("Start Async", || log_error!(Self::start_async()))?;
        std::thread::spawn(|| log_error!(Self::refresh_auto_start_path()));
        Ok(())
    }

    /// Stop and release all resources
    pub fn stop(&self) {
        release_system_events_handlers();
        if self.state().is_weg_enabled() {
            log_error!(SeelenWeg::show_taskbar());
        }
        if self.state().is_ahk_enabled() {
            Self::kill_ahk_shortcuts();
        }
    }

    fn add_monitor(&mut self, hmonitor: HMONITOR) -> Result<()> {
        self.monitors.push(Monitor::new(hmonitor, &self.state())?);
        Ok(())
    }

    fn remove_monitor(&mut self, hmonitor: HMONITOR) -> Result<()> {
        self.monitors.retain(|m| m.handle() != &hmonitor);
        Ok(())
    }

    fn ensure_folders(handle: &AppHandle<Wry>) -> Result<()> {
        log::trace!("Ensuring folders");
        let path = handle.path();
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
        std::fs::create_dir_all(data_path.join("wallpapers"))?;

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

    /// override auto-start task in case of location change, normally this happen on MSIX update
    fn refresh_auto_start_path() -> Result<()> {
        if WindowsApi::is_elevated()? && Self::is_auto_start_enabled()? {
            Self::set_auto_start(true)?;
        }
        Ok(())
    }

    pub fn set_auto_start(enabled: bool) -> Result<()> {
        let pwsh_script = include_str!("schedule.ps1");
        let pwsh_script_path = temp_dir().join("schedule.ps1");
        std::fs::write(&pwsh_script_path, pwsh_script).expect("Failed to write temp script file");

        let exe_path = std::env::current_exe()?;

        tauri::async_runtime::block_on(async move {
            let result = get_app_handle()
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

    pub fn start_ahk_shortcuts() -> Result<()> {
        // kill all running shortcuts before starting again
        Self::kill_ahk_shortcuts();

        let state = FULL_STATE.load();
        if state.is_ahk_enabled() {
            log::trace!("Starting AHK shortcuts");
            AutoHotKey::new(include_str!("utils/ahk/mocks/seelen.ahk"))
                .with_lib()
                .execute()?;

            if state.is_window_manager_enabled() {
                log::trace!("Starting seelen.wm.ahk");
                AutoHotKey::from_template(
                    include_str!("utils/ahk/mocks/seelen.wm.ahk"),
                    state.get_ahk_variables(),
                )
                .with_lib()
                .execute()?;
            }
        }
        log::trace!("AHK shortcuts started successfully");
        Ok(())
    }

    pub fn kill_ahk_shortcuts() {
        log::trace!("Killing AHK shortcuts");
        get_app_handle()
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

    pub fn show_settings() -> Result<()> {
        log::trace!("Show settings window");
        let handle = get_app_handle();
        let window = handle.get_webview_window("settings").or_else(|| {
            tauri::WebviewWindowBuilder::new(
                &handle,
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
            .always_on_top(true)
            .build()
            .ok()
        });

        match window {
            Some(window) => {
                window.unminimize()?;
                window.set_focus()?;
                Ok(())
            }
            None => Err("Failed to create settings window".into()),
        }
    }

    pub fn show_update_modal() -> Result<()> {
        log::trace!("Showing update notification window");
        let handle = get_app_handle();
        // check if path is in windows apps folder
        let installation_path = handle.path().resource_dir()?;
        if installation_path
            .to_string_lossy()
            .contains(r"\Program Files\WindowsApps\")
        {
            log::trace!("Skipping update notification because it is installed as MSIX");
            return Ok(());
        }

        tauri::WebviewWindowBuilder::new(
            &handle,
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
    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, _: LPARAM) -> BOOL {
        let mut seelen = trace_lock!(SEELEN);

        if SeelenWeg::is_real_window(hwnd, false) {
            SeelenWeg::add_hwnd(hwnd);
        }

        for monitor in seelen.monitors_mut() {
            if let Some(wm) = monitor.wm_mut() {
                if WindowManager::is_manageable_window(hwnd, true) {
                    log_error!(wm.add_hwnd(hwnd));
                }
            }
        }
        true.into()
    }
}
