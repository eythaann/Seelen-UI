use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, LazyLock},
};

use parking_lot::Mutex;
use seelen_core::{handlers::SeelenEvent, system_state::MonitorId};
use slu_ipc::messages::SvcAction;
use tauri::{AppHandle, Listener, Wry};
use windows::Win32::System::TaskScheduler::{ITaskService, TaskScheduler};

use crate::{
    app_instance::SluMonitorInstance,
    cli::ServicePipe,
    error::{Result, ResultLogExt},
    hook::register_win_hook,
    log_error,
    modules::{
        monitors::{MonitorManager, MonitorManagerEvent},
        system_settings::application::{SystemSettings, SystemSettingsEvent},
    },
    restoration_and_migrations::RestorationAndMigration,
    state::application::{FullState, FULL_STATE},
    system::{declare_system_events_handlers, release_system_events_handlers},
    trace_lock,
    utils::discord::start_discord_rpc,
    virtual_desktops::get_vd_manager,
    widgets::{
        launcher::SeelenRofi,
        loader::WidgetInstance,
        task_switcher::TaskSwitcher,
        wallpaper_manager::SeelenWall,
        weg::{weg_items_impl::SEELEN_WEG_STATE, SeelenWeg},
        window_manager::instance::WindowManagerV2,
    },
    windows_api::{event_window::create_background_window, monitor::MonitorView, Com},
    APP_HANDLE,
};

pub static SEELEN: LazyLock<Arc<Mutex<Seelen>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Seelen::default())));

static SEELEN_IS_RUNNING: AtomicBool = AtomicBool::new(false);

/// Tauri app handle
pub fn get_app_handle<'a>() -> &'a AppHandle<Wry> {
    APP_HANDLE
        .get()
        .expect("get_app_handle called but app is still not initialized")
}

/** Struct should be initialized first before calling any other methods */
#[derive(Default)]
pub struct Seelen {
    pub instances: Vec<SluMonitorInstance>,
    pub wall: Option<SeelenWall>,
    pub rofi: Option<SeelenRofi>,
    pub task_switcher: Option<TaskSwitcher>,
    #[allow(dead_code)]
    pub widgets: HashMap<String, WidgetInstance>,
}

/* ============== Getters ============== */
impl Seelen {
    pub fn instances_mut(&mut self) -> &mut Vec<SluMonitorInstance> {
        &mut self.instances
    }

    pub fn is_running() -> bool {
        SEELEN_IS_RUNNING.load(std::sync::atomic::Ordering::Acquire)
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
            log_error!(wall.update_position());
            self.wall = Some(wall)
        }
        Ok(())
    }

    fn add_task_switcher(&mut self) -> Result<()> {
        if self.task_switcher.is_none() {
            self.task_switcher = Some(TaskSwitcher::new()?);
        }
        Ok(())
    }

    fn refresh_windows_positions(&mut self) -> Result<()> {
        if let Some(wall) = &self.wall {
            wall.update_position()?;
        }
        for instance in &mut self.instances {
            instance.ensure_positions()?;
        }
        Ok(())
    }

    pub fn on_settings_change(&mut self, state: &FullState) -> Result<()> {
        rust_i18n::set_locale(state.locale());
        ServicePipe::request(SvcAction::SetSettings(Box::new(state.settings.clone())))?;

        if state.is_weg_enabled() {
            SeelenWeg::hide_taskbar();
        } else {
            SeelenWeg::restore_taskbar()?;
        }

        match state.is_window_manager_enabled() {
            true => {
                WindowManagerV2::init_state();
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
            monitor.load_settings(state)?;
        }

        self.add_task_switcher()?;
        self.refresh_windows_positions()?;
        Ok(())
    }

    fn on_monitor_event(event: MonitorManagerEvent) {
        let mut guard = trace_lock!(SEELEN);
        match event {
            MonitorManagerEvent::ViewAdded(view) => {
                log_error!(guard.add_monitor(view));
            }
            MonitorManagerEvent::ViewRemoved(id) => {
                log_error!(guard.remove_monitor(&id));
            }
        }
    }

    fn on_system_settings_change(event: SystemSettingsEvent) {
        if event == SystemSettingsEvent::TextScaleChanged {
            log_error!(trace_lock!(SEELEN).refresh_windows_positions());
        }
    }

    pub fn start(&mut self) -> Result<()> {
        RestorationAndMigration::run_full()?;

        let state = FULL_STATE.load();
        rust_i18n::set_locale(state.locale());

        // order is important
        create_background_window()?;
        declare_system_events_handlers()?;
        self.add_task_switcher()?;

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
        for view in MonitorManager::get_all_views()? {
            self.add_monitor(view)?;
        }

        MonitorManager::subscribe(Self::on_monitor_event);
        SystemSettings::subscribe(Self::on_system_settings_change);

        self.refresh_windows_positions()?;

        get_vd_manager().list_windows_into_respective_workspace()?;
        if state.is_window_manager_enabled() {
            WindowManagerV2::enumerate_all_windows()?;
        }

        register_win_hook()?;
        start_discord_rpc()?;

        if state.are_shortcuts_enabled() {
            ServicePipe::request(SvcAction::SetSettings(Box::new(state.settings.clone())))?;
        }

        get_app_handle().listen(SeelenEvent::StateWidgetsChanged, |_| {
            std::thread::spawn(|| {
                let mut guard = trace_lock!(SEELEN);
                for monitor in &mut guard.instances {
                    monitor.reload_widgets().log_error();
                }
            });
        });

        SEELEN_IS_RUNNING.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// Stop and release all resources
    pub fn stop(&self) {
        SEELEN_IS_RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
        release_system_events_handlers();
    }

    fn add_monitor(&mut self, view: MonitorView) -> Result<()> {
        let state = FULL_STATE.load();
        self.instances.push(SluMonitorInstance::new(view, &state)?);
        self.refresh_windows_positions()?;
        trace_lock!(SEELEN_WEG_STATE).emit_to_webview()?;
        Ok(())
    }

    fn remove_monitor(&mut self, id: &MonitorId) -> Result<()> {
        self.instances.retain(|m| &m.main_target_id != id);
        self.refresh_windows_positions()?;
        trace_lock!(SEELEN_WEG_STATE).emit_to_webview()?;
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
        ServicePipe::request(SvcAction::SetStartup(enabled))
    }
}
