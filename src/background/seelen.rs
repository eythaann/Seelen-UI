use std::{
    env::temp_dir,
    sync::{atomic::AtomicBool, Arc},
};

use base64::Engine;
use getset::{Getters, MutGetters};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::handlers::SeelenEvent;
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::Graphics::Gdi::HMONITOR;
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS},
    RegKey,
};

use crate::{
    error_handler::Result,
    hook::register_win_hook,
    instance::SeelenInstanceContainer,
    log_error,
    modules::monitors::{MonitorManagerEvent, MONITOR_MANAGER},
    restoration_and_migrations::RestorationAndMigration,
    seelen_rofi::SeelenRofi,
    seelen_wall::SeelenWall,
    seelen_weg::SeelenWeg,
    seelen_wm_v2::instance::WindowManagerV2,
    state::application::{FullState, FULL_STATE},
    system::{declare_system_events_handlers, release_system_events_handlers},
    trace_lock,
    utils::{ahk::AutoHotKey, is_msix_intallation, PERFORMANCE_HELPER},
    windows_api::WindowsApi,
    APP_HANDLE,
};

lazy_static! {
    pub static ref SEELEN: Arc<Mutex<Seelen>> = Arc::new(Mutex::new(Seelen::default()));
}

static SEELEN_IS_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn get_app_handle<'a>() -> &'a AppHandle<Wry> {
    APP_HANDLE
        .get()
        .expect("get_app_handle called but app is still not initialized")
}

/** Struct should be initialized first before calling any other methods */
#[derive(Getters, MutGetters, Default)]
pub struct Seelen {
    #[getset(get = "pub", get_mut = "pub")]
    monitors: Vec<SeelenInstanceContainer>,
    #[getset(get = "pub", get_mut = "pub")]
    rofi: Option<SeelenRofi>,
    #[getset(get = "pub", get_mut = "pub")]
    wall: Option<SeelenWall>,
}

/* ============== Getters ============== */
impl Seelen {
    pub fn is_running() -> bool {
        SEELEN_IS_RUNNING.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn focused_monitor(&self) -> Option<&SeelenInstanceContainer> {
        self.monitors.iter().find(|m| m.is_focused())
    }

    pub fn focused_monitor_mut(&mut self) -> Option<&mut SeelenInstanceContainer> {
        self.monitors.iter_mut().find(|m| m.is_focused())
    }

    pub fn monitor_by_id_mut(&mut self, id: isize) -> Option<&mut SeelenInstanceContainer> {
        self.monitors
            .iter_mut()
            .find(|m| m.handle().0 as isize == id)
    }

    pub fn monitor_by_name_mut(&mut self, name: &str) -> Option<&mut SeelenInstanceContainer> {
        self.monitors.iter_mut().find(|m| m.name() == name)
    }

    pub fn state(&self) -> Arc<FullState> {
        FULL_STATE.load_full()
    }
}

/* ============== Methods ============== */
impl Seelen {
    fn add_rofi(&mut self) -> Result<()> {
        if self.rofi.is_none() {
            self.rofi = Some(SeelenRofi::new()?);
        }
        Ok(())
    }

    fn add_wall(&mut self) -> Result<()> {
        if self.wall.is_none() {
            let wall = SeelenWall::new()?;
            wall.update_position()?;
            self.wall = Some(wall)
        }
        Ok(())
    }

    fn refresh_windows_positions(&mut self) -> Result<()> {
        if let Some(wall) = &self.wall {
            wall.update_position()?;
        }
        for monitor in &mut self.monitors {
            if WindowsApi::monitor_info(*monitor.handle()).is_ok() {
                monitor.ensure_positions()?;
            }
        }
        Ok(())
    }

    pub fn on_settings_change(&mut self) -> Result<()> {
        let state = self.state();

        match state.is_ahk_enabled() {
            true => Self::start_ahk_shortcuts()?,
            false => Self::kill_ahk_shortcuts()?,
        }

        if state.is_weg_enabled() {
            SeelenWeg::hide_taskbar();
        } else {
            SeelenWeg::restore_taskbar()?;
        }

        match state.is_window_manager_enabled() {
            true => {
                WindowManagerV2::init_state()?;
                WindowManagerV2::enumerate_all_windows()?;
            }
            false => WindowManagerV2::clear_state(),
        }

        match state.is_rofi_enabled() {
            true => self.add_rofi()?,
            false => self.rofi = None,
        }

        match state.is_wall_enabled() {
            true => self.add_wall()?,
            false => self.wall = None,
        }

        for monitor in &mut self.monitors {
            monitor.load_settings(&state)?;
        }

        self.refresh_windows_positions()?;
        Ok(())
    }

    fn on_monitor_event(event: MonitorManagerEvent) {
        match event {
            MonitorManagerEvent::Added(_name, id) => {
                log_error!(trace_lock!(SEELEN).add_monitor(id));
            }
            MonitorManagerEvent::Removed(_name, id) => {
                log_error!(trace_lock!(SEELEN).remove_monitor(id));
            }
            MonitorManagerEvent::Updated(name, id) => {
                if let Some(m) = trace_lock!(SEELEN).monitor_by_name_mut(&name) {
                    m.update_handle(id);
                }
            }
        }
        log_error!(get_app_handle().emit(SeelenEvent::GlobalMonitorsChanged, ()));
    }

    async fn start_async() -> Result<()> {
        if FULL_STATE.load().is_weg_enabled() {
            SeelenWeg::enumerate_all_windows()?;
        }

        if FULL_STATE.load().is_window_manager_enabled() {
            WindowManagerV2::enumerate_all_windows()?;
        }

        Self::start_ahk_shortcuts()?;
        Self::refresh_path_environment()?;
        Self::refresh_auto_start_path().await?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        SEELEN_IS_RUNNING.store(true, std::sync::atomic::Ordering::SeqCst);
        RestorationAndMigration::run_full()?;
        declare_system_events_handlers()?;

        if self.state().is_rofi_enabled() {
            self.add_rofi()?;
        }

        if self.state().is_wall_enabled() {
            self.add_wall()?;
        }

        if self.state().is_weg_enabled() {
            SeelenWeg::hide_taskbar();
        }

        log::trace!("Enumerating Monitors & Creating Instances");
        let monitors = trace_lock!(MONITOR_MANAGER).monitors.clone();
        for (_name, id) in monitors {
            self.add_monitor(id)?;
        }
        trace_lock!(MONITOR_MANAGER).listen_changes(Self::on_monitor_event);

        tauri::async_runtime::spawn(async {
            trace_lock!(PERFORMANCE_HELPER).start("lazy setup");
            log_error!(Self::start_async().await);
            trace_lock!(PERFORMANCE_HELPER).end("lazy setup");
        });

        self.refresh_windows_positions()?;
        register_win_hook()?;
        Ok(())
    }

    /// Stop and release all resources
    pub fn stop(&self) {
        SEELEN_IS_RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);

        release_system_events_handlers();
        if self.state().is_weg_enabled() {
            log_error!(SeelenWeg::restore_taskbar());
        }
        if self.state().is_ahk_enabled() {
            log_error!(Self::kill_ahk_shortcuts());
        }
    }

    fn add_monitor(&mut self, hmonitor: HMONITOR) -> Result<()> {
        self.monitors
            .push(SeelenInstanceContainer::new(hmonitor, &self.state())?);
        self.refresh_windows_positions()?;
        Ok(())
    }

    fn remove_monitor(&mut self, hmonitor: HMONITOR) -> Result<()> {
        self.monitors.retain(|m| m.handle() != &hmonitor);
        self.refresh_windows_positions()?;
        Ok(())
    }

    pub async fn is_auto_start_enabled() -> Result<bool> {
        let handle = get_app_handle();
        let output = handle
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
            .await?;

        let (cow, _used, _has_errors) = encoding_rs::GBK.decode(&output.stdout);
        let stdout = cow.to_string().trim().to_lowercase();
        Ok(stdout == "true")
    }

    /// override auto-start task in case of location change, normally this happen on MSIX update
    async fn refresh_auto_start_path() -> Result<()> {
        if WindowsApi::is_elevated()? && Self::is_auto_start_enabled().await? {
            Self::set_auto_start(true).await?;
        }
        Ok(())
    }

    pub fn refresh_path_environment() -> Result<()> {
        if tauri::is_dev() || is_msix_intallation() {
            return Ok(());
        }

        let hkcr = RegKey::predef(HKEY_CURRENT_USER);
        let enviroments = hkcr.open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
        let env_path: String = enviroments.get_value("Path")?;

        let install_folder = std::env::current_exe()?
            .parent()
            .expect("Failed to get parent directory")
            .to_string_lossy()
            .to_string();
        if !env_path.contains(&install_folder) {
            log::trace!("Adding installation directory to PATH environment variable");
            enviroments.set_value("Path", &format!("{};{}", env_path, install_folder))?;
        }
        Ok(())
    }

    pub async fn set_auto_start(enabled: bool) -> Result<()> {
        let pwsh_script = include_str!("schedule.ps1");
        let pwsh_script_path = temp_dir().join("schedule.ps1");
        std::fs::write(&pwsh_script_path, pwsh_script).expect("Failed to write temp script file");

        let exe_path = std::env::current_exe()?;

        let output = get_app_handle()
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
            .await?;

        if !output.status.success() {
            return Err(output.into());
        }
        Ok(())
    }

    // TODO: split ahk logic into another file/module
    pub fn start_ahk_shortcuts() -> Result<()> {
        // kill all running shortcuts before starting again
        Self::kill_ahk_shortcuts()?;

        let state = FULL_STATE.load();
        if state.is_ahk_enabled() {
            log::trace!("Creating AHK shortcuts");
            let vars = state.get_ahk_variables();

            AutoHotKey::new(include_str!("utils/ahk/mocks/seelen.lib.ahk"))
                .name("seelen.lib.ahk")
                .save()?;

            AutoHotKey::from_template(include_str!("utils/ahk/mocks/seelen.ahk"), &vars)
                .name("seelen.ahk")
                .execute()?;

            AutoHotKey::from_template(include_str!("utils/ahk/mocks/seelen.vd.ahk"), &vars)
                .name("seelen.vd.ahk")
                .execute()?;

            if state.is_window_manager_enabled() {
                AutoHotKey::from_template(include_str!("utils/ahk/mocks/seelen.wm.ahk"), &vars)
                    .name("seelen.wm.ahk")
                    .execute()?;
            }

            if state.is_rofi_enabled() {
                AutoHotKey::from_template(
                    include_str!("utils/ahk/mocks/seelen.launcher.ahk"),
                    &vars,
                )
                .name("seelen.launcher.ahk")
                .execute()?;
            }
        }
        log::trace!("AHK shortcuts started successfully");
        Ok(())
    }

    // TODO: split ahk logic into another file/module
    pub fn kill_ahk_shortcuts() -> Result<()> {
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
            .spawn()?;
        Ok(())
    }

    pub fn show_settings() -> Result<()> {
        log::trace!("Show settings window");
        let label = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("@seelen/settings");
        let handle = get_app_handle();
        let window = handle.get_webview_window(&label).or_else(|| {
            tauri::WebviewWindowBuilder::new(
                handle,
                label,
                tauri::WebviewUrl::App("settings/index.html".into()),
            )
            .title("Settings")
            .inner_size(750.0, 490.0)
            .min_inner_size(600.0, 400.0)
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
                Ok(())
            }
            None => Err("Failed to create settings window".into()),
        }
    }
}
