use std::{
    env::temp_dir,
    sync::{atomic::AtomicBool, Arc, OnceLock},
};

use getset::{Getters, MutGetters};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{path::BaseDirectory, AppHandle, Manager, Wry};
use tauri_plugin_shell::ShellExt;
use windows::Win32::Graphics::Gdi::HMONITOR;

use crate::{
    error_handler::Result,
    hook::register_win_hook,
    instance::SeelenInstanceContainer,
    log_error,
    modules::monitors::{MonitorManagerEvent, MONITOR_MANAGER},
    seelen_rofi::SeelenRofi,
    seelen_wall::SeelenWall,
    seelen_weg::SeelenWeg,
    seelen_wm_v2::instance::WindowManagerV2,
    state::application::{FullState, FULL_STATE},
    system::{declare_system_events_handlers, release_system_events_handlers},
    trace_lock,
    utils::{ahk::AutoHotKey, PERFORMANCE_HELPER},
    windows_api::WindowsApi,
};

lazy_static! {
    pub static ref SEELEN: Arc<Mutex<Seelen>> = Arc::new(Mutex::new(Seelen::default()));
}

static APP_HANDLE: OnceLock<AppHandle<Wry>> = OnceLock::new();
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
            monitor.ensure_positions()?;
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

    /// Initialize Seelen and Lazy static variables
    pub fn init(&mut self, handle: &AppHandle<Wry>) -> Result<()> {
        log::trace!("Initializing Seelen");
        APP_HANDLE
            .set(handle.to_owned())
            .map_err(|_| "Failed to set app handle")?;
        Self::ensure_folders(handle)?;
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

    async fn start_async() -> Result<()> {
        if FULL_STATE.load().is_weg_enabled() {
            SeelenWeg::enumerate_all_windows()?;
        }

        if FULL_STATE.load().is_window_manager_enabled() {
            WindowManagerV2::enumerate_all_windows()?;
        }

        Self::start_ahk_shortcuts()?;
        Self::refresh_auto_start_path().await?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        SEELEN_IS_RUNNING.store(true, std::sync::atomic::Ordering::SeqCst);
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

        let create_if_needed = move |folder: &str| -> Result<()> {
            let path = data_path.join(folder);
            if !path.exists() {
                std::fs::create_dir_all(path)?;
            }
            Ok(())
        };

        create_if_needed("placeholders")?;
        create_if_needed("themes")?;
        create_if_needed("layouts")?;
        create_if_needed("icons/system")?;
        create_if_needed("wallpapers")?;

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

    pub fn start_ahk_shortcuts() -> Result<()> {
        // kill all running shortcuts before starting again
        Self::kill_ahk_shortcuts()?;

        let state = FULL_STATE.load();
        if state.is_ahk_enabled() {
            log::trace!("Creating AHK shortcuts");

            AutoHotKey::new(include_str!("utils/ahk/mocks/seelen.lib.ahk"))
                .name("seelen.lib.ahk")
                .save()?;

            AutoHotKey::new(include_str!("utils/ahk/mocks/seelen.ahk"))
                .name("seelen.ahk")
                .execute()?;

            AutoHotKey::from_template(
                include_str!("utils/ahk/mocks/seelen.vd.ahk"),
                state.get_ahk_variables(),
            )
            .name("seelen.vd.ahk")
            .execute()?;

            if state.is_window_manager_enabled() {
                AutoHotKey::from_template(
                    include_str!("utils/ahk/mocks/seelen.wm.ahk"),
                    state.get_ahk_variables(),
                )
                .name("seelen.wm.ahk")
                .execute()?;
            }
        }
        log::trace!("AHK shortcuts started successfully");
        Ok(())
    }

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
        let handle = get_app_handle();
        let window = handle.get_webview_window("settings").or_else(|| {
            tauri::WebviewWindowBuilder::new(
                handle,
                "settings",
                tauri::WebviewUrl::App("settings/index.html".into()),
            )
            .title("Settings")
            .inner_size(750.0, 480.0)
            .min_inner_size(700.0, 400.0)
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
