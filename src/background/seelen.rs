use std::sync::{atomic::AtomicBool, Arc};

use base64::Engine;
use getset::{Getters, MutGetters};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{AppHandle, Manager, Wry};
use windows::Win32::{
    Graphics::Gdi::HMONITOR,
    System::TaskScheduler::{ITaskService, TaskScheduler},
};

use crate::{
    error_handler::Result,
    hook::register_win_hook,
    instance::SeelenInstanceContainer,
    log_error,
    modules::{
        cli::{SvcAction, TcpService},
        monitors::{MonitorManager, MonitorManagerEvent, MONITOR_MANAGER},
        system_settings::application::{SystemSettings, SystemSettingsEvent},
    },
    restoration_and_migrations::RestorationAndMigration,
    seelen_rofi::SeelenRofi,
    seelen_wall::SeelenWall,
    seelen_weg::{weg_items_impl::WEG_ITEMS_IMPL, SeelenWeg},
    seelen_wm_v2::instance::WindowManagerV2,
    state::application::FULL_STATE,
    system::{declare_system_events_handlers, release_system_events_handlers},
    trace_lock,
    utils::{ahk::AutoHotKey, pwsh::PwshScript},
    windows_api::{event_window::create_background_window, Com, WindowsApi},
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
    instances: Vec<SeelenInstanceContainer>,
    #[getset(get = "pub", get_mut = "pub")]
    rofi: Option<SeelenRofi>,
    #[getset(get = "pub", get_mut = "pub")]
    wall: Option<SeelenWall>,
}

/* ============== Getters ============== */
impl Seelen {
    pub fn instances(&self) -> &Vec<SeelenInstanceContainer> {
        &self.instances
    }

    pub fn instances_mut(&mut self) -> &mut Vec<SeelenInstanceContainer> {
        &mut self.instances
    }

    pub fn is_running() -> bool {
        SEELEN_IS_RUNNING.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn focused_monitor(&self) -> Option<&SeelenInstanceContainer> {
        self.instances.iter().find(|m| m.is_focused())
    }

    pub fn focused_monitor_mut(&mut self) -> Option<&mut SeelenInstanceContainer> {
        self.instances.iter_mut().find(|m| m.is_focused())
    }

    pub fn monitor_by_device_id_mut(&mut self, id: &str) -> Option<&mut SeelenInstanceContainer> {
        self.instances.iter_mut().find(|m| m.id() == id)
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
        for instance in &mut self.instances {
            if WindowsApi::monitor_info(instance.monitor().handle()).is_ok() {
                instance.ensure_positions()?;
            }
        }
        Ok(())
    }

    pub fn on_settings_change(&mut self) -> Result<()> {
        let state = FULL_STATE.load();

        tauri::async_runtime::spawn(async {
            let state = FULL_STATE.load();
            match state.is_ahk_enabled() {
                true => log_error!(Self::start_ahk_shortcuts().await),
                false => log_error!(Self::kill_ahk_shortcuts().await),
            }
        });

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

        for monitor in &mut self.instances {
            monitor.load_settings(&state)?;
        }

        self.refresh_windows_positions()?;
        Ok(())
    }

    fn on_monitor_event(event: MonitorManagerEvent) {
        match event {
            MonitorManagerEvent::Added(_id, handle) => {
                log_error!(trace_lock!(SEELEN).add_monitor(handle));
            }
            MonitorManagerEvent::Removed(id, _handle) => {
                log_error!(trace_lock!(SEELEN).remove_monitor(&id));
            }
            MonitorManagerEvent::Updated(id, handle) => {
                if let Some(m) = trace_lock!(SEELEN).monitor_by_device_id_mut(&id) {
                    m.update_handle(handle);
                }
                log_error!(trace_lock!(SEELEN).refresh_windows_positions());
                log_error!(trace_lock!(WEG_ITEMS_IMPL).emit_to_webview());
            }
        }
    }

    fn on_system_settings_change(event: SystemSettingsEvent) {
        if event == SystemSettingsEvent::TextScaleChanged {
            log_error!(trace_lock!(SEELEN).refresh_windows_positions());
        }
    }

    async fn start_async() -> Result<()> {
        Self::start_ahk_shortcuts().await?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        SEELEN_IS_RUNNING.store(true, std::sync::atomic::Ordering::SeqCst);
        RestorationAndMigration::run_full()?;

        // order is important
        create_background_window()?;
        declare_system_events_handlers()?;

        let state = FULL_STATE.load();

        if state.is_rofi_enabled() {
            self.add_rofi()?;
        }

        if state.is_wall_enabled() {
            self.add_wall()?;
        }

        if state.is_weg_enabled() {
            SeelenWeg::hide_taskbar();
        }

        log::trace!("Enumerating Monitors & Creating Instances");
        let monitors = { trace_lock!(MONITOR_MANAGER).monitors.clone() };
        for (_name, id) in monitors {
            self.add_monitor(id)?;
        }

        MonitorManager::subscribe(Self::on_monitor_event);
        SystemSettings::subscribe(Self::on_system_settings_change);

        tauri::async_runtime::spawn(async {
            log_error!(Self::start_async().await);
        });

        self.refresh_windows_positions()?;

        if FULL_STATE.load().is_weg_enabled() {
            SeelenWeg::enumerate_all_windows()?;
        }

        if FULL_STATE.load().is_window_manager_enabled() {
            WindowManagerV2::enumerate_all_windows()?;
        }

        register_win_hook()?;
        Ok(())
    }

    /// Stop and release all resources
    pub fn stop(&self) {
        SEELEN_IS_RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
        release_system_events_handlers();
        let state = FULL_STATE.load();
        if state.is_ahk_enabled() {
            tauri::async_runtime::spawn(async {
                log_error!(Self::kill_ahk_shortcuts().await);
            });
        }
    }

    fn add_monitor(&mut self, handle: HMONITOR) -> Result<()> {
        let state = FULL_STATE.load();
        self.instances
            .push(SeelenInstanceContainer::new(handle, &state)?);
        self.refresh_windows_positions()?;
        trace_lock!(WEG_ITEMS_IMPL).emit_to_webview()?;
        Ok(())
    }

    fn remove_monitor(&mut self, id: &str) -> Result<()> {
        self.instances.retain(|m| m.id() != id);
        self.refresh_windows_positions()?;
        trace_lock!(WEG_ITEMS_IMPL).emit_to_webview()?;
        Ok(())
    }

    pub fn is_auto_start_enabled() -> Result<bool> {
        Com::run_with_context(|| unsafe {
            let task_service: ITaskService = Com::create_instance(&TaskScheduler)?;
            task_service.Connect(
                &Default::default(),
                &Default::default(),
                &Default::default(),
                &Default::default(),
            )?;
            let is_task_enabled = task_service
                .GetFolder(&"\\Seelen".into())
                .and_then(|folder| folder.GetTask(&"Seelen UI Service".into()))
                .and_then(|task| task.Definition())
                .and_then(|definition| definition.Triggers())
                .and_then(|triggers| triggers.get_Item(1))
                .is_ok();
            Ok(is_task_enabled)
        })
    }

    pub fn set_auto_start(enabled: bool) -> Result<()> {
        TcpService::request(SvcAction::SetStartup(enabled))
    }

    // TODO: split ahk logic into another file/module
    pub async fn start_ahk_shortcuts() -> Result<()> {
        // kill all running shortcuts before starting again
        Self::kill_ahk_shortcuts().await?;

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

            if state.is_weg_enabled() {
                AutoHotKey::from_template(include_str!("utils/ahk/mocks/seelen.weg.ahk"), &vars)
                    .name("seelen.weg.ahk")
                    .execute()?;
            }

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
    pub async fn kill_ahk_shortcuts() -> Result<()> {
        log::trace!("Killing AHK shortcuts");
        PwshScript::new(
            r"Get-WmiObject Win32_Process | Where-Object { $_.CommandLine -like '*static\redis\AutoHotkey.exe*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }"
        ).execute().await?;
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
