use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock,
};

use parking_lot::Mutex;
use seelen_core::system_state::MonitorId;
use slu_ipc::messages::SvcAction;
use tauri::{AppHandle, Emitter, Wry};
use windows::Win32::System::TaskScheduler::{ITaskService, TaskScheduler};

use crate::{
    app_instance::LegacyWidgetMonitorContainer,
    cli::ServicePipe,
    error::{Result, ResultLogExt},
    hook::register_win_hook,
    log_error,
    migrations::Migrations,
    modules::{
        monitors::{MonitorManager, MonitorManagerEvent},
        system_settings::application::{SystemSettings, SystemSettingsEvent},
    },
    state::application::{FullState, FULL_STATE},
    trace_lock,
    utils::discord::start_discord_rpc,
    widgets::{
        wallpaper_manager::SeelenWall,
        weg::{weg_items_impl::SEELEN_WEG_STATE, SeelenWeg},
    },
    windows_api::{
        event_window::{create_background_window, IS_INTERACTIVE_SESSION},
        Com,
    },
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

pub fn emit_to_webviews<S>(event: &str, payload: S)
where
    S: serde::Serialize + Clone,
{
    // log::trace!("Emitting {event} to webviews");
    if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
        // log::debug!("Skipping event {event} because session is not active");
        return;
    }
    get_app_handle().emit(event, payload).log_error();
}

/** Struct should be initialized first before calling any other methods */
#[derive(Default)]
pub struct Seelen {
    pub widgets_per_display: Vec<LegacyWidgetMonitorContainer>,
    pub wall: Option<SeelenWall>,
}

/* ============== Getters ============== */
impl Seelen {
    pub fn instances_mut(&mut self) -> &mut Vec<LegacyWidgetMonitorContainer> {
        &mut self.widgets_per_display
    }

    pub fn is_running() -> bool {
        SEELEN_IS_RUNNING.load(std::sync::atomic::Ordering::Acquire)
    }
}

/* ============== Methods ============== */
impl Seelen {
    fn add_wall(&mut self) -> Result<()> {
        if self.wall.is_none() {
            let wall = SeelenWall::new()?;
            log_error!(wall.update_position());
            self.wall = Some(wall)
        }
        Ok(())
    }

    fn refresh_windows_positions(&mut self) -> Result<()> {
        if let Some(wall) = &self.wall {
            wall.update_position()?;
        }
        for instance in &mut self.widgets_per_display {
            instance.ensure_positions()?;
        }
        Ok(())
    }

    pub fn on_settings_change(&mut self, state: &FullState) -> Result<()> {
        rust_i18n::set_locale(state.locale());
        ServicePipe::request(SvcAction::SetSettings(Box::new(state.settings.clone())))?;

        if state.is_weg_enabled() {
            SeelenWeg::hide_native_taskbar();
        } else {
            SeelenWeg::restore_native_taskbar()?;
        }

        match state.is_wall_enabled() {
            true => self.add_wall()?,
            false => self.wall = None,
        }

        for monitor in &mut self.widgets_per_display {
            monitor.load_settings(state)?;
        }

        self.refresh_windows_positions()?;
        Ok(())
    }

    fn on_monitor_event(event: MonitorManagerEvent) {
        let mut guard = trace_lock!(SEELEN);
        match event {
            MonitorManagerEvent::ViewAdded(id) => {
                log_error!(guard.add_monitor(id));
            }
            MonitorManagerEvent::ViewsChanged => {
                log_error!(guard.refresh_windows_positions());
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
        Migrations::run()?;

        let state = FULL_STATE.load();
        rust_i18n::set_locale(state.locale());

        if state.is_wall_enabled() {
            self.add_wall()?;
        }

        log::trace!("Enumerating Monitors & Creating Instances");
        for view in MonitorManager::instance().read_all_views()? {
            self.add_monitor(view.primary_target()?.stable_id()?)?;
        }

        self.refresh_windows_positions()?;

        create_background_window()?;
        register_win_hook()?;
        MonitorManager::subscribe(Self::on_monitor_event);
        SystemSettings::subscribe(Self::on_system_settings_change);

        start_discord_rpc()?;
        ServicePipe::request(SvcAction::SetSettings(Box::new(state.settings.clone())))?;
        SEELEN_IS_RUNNING.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    /// Stop and release all resources
    pub fn stop(&self) {
        SEELEN_IS_RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    fn add_monitor(&mut self, monitor_id: MonitorId) -> Result<()> {
        let state = FULL_STATE.load();
        self.widgets_per_display
            .push(LegacyWidgetMonitorContainer::new(monitor_id, &state)?);
        self.refresh_windows_positions()?;
        // why this is here? TODO: refactor this.
        trace_lock!(SEELEN_WEG_STATE).emit_to_webview()?;
        Ok(())
    }

    fn remove_monitor(&mut self, id: &MonitorId) -> Result<()> {
        self.widgets_per_display
            .retain(|m| &m.view_primary_target_id != id);
        self.refresh_windows_positions()?;
        // why this is here? TODO: refactor this.
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
