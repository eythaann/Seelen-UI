use std::sync::atomic::Ordering;

use slu_ipc::messages::SvcAction;
use tauri::{AppHandle, Emitter, Wry};
use windows::Win32::System::TaskScheduler::{ITaskService, TaskScheduler};

use seelen_core::state::shortcuts::resolve_shortcuts;

use crate::{
    APP_HANDLE,
    cli::ServicePipe,
    error::{Result, ResultLogExt},
    hook::register_win_hook,
    migrations::Migrations,
    modules::{
        clipboard::infrastructure::init_clipboard_manager, start::application::StartMenuManager,
        user::infrastructure::reemit_user,
    },
    resources::RESOURCES,
    session::infrastructure::reemit_session,
    state::application::{AppSettings, FULL_STATE, initialize_user_resources_watcher},
    utils::discord::{start_discord_rpc, update_discord_rpc},
    widgets::{
        manager::WIDGET_MANAGER, popups::shortcut_conflicts::show_shortcut_conflict_popup,
        weg::SeelenWeg,
    },
    windows_api::{
        Com,
        event_window::{IS_INTERACTIVE_SESSION, create_background_window},
    },
};

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

pub struct SeelenUI {}

/* ============== Methods ============== */
impl SeelenUI {
    /// this fn prepares the app loading assets and settings in memory parallelly
    pub async fn pre_start() -> Result<()> {
        Migrations::run_all()?;

        let webview_check = tokio::task::spawn(async {
            crate::measure!(
                "Webview optimal state",
                crate::utils::integrity::check_for_webview_optimal_state().await
            )
        });

        crate::measure!("RESOURCES", RESOURCES.initialize().await);
        crate::measure!("START_MENU", StartMenuManager::initialize().log_error());
        crate::measure!("APP SETTINGS", FULL_STATE.load());

        if !webview_check.await.unwrap_or(false) {
            return Err("Webview optimal state check failed".into());
        }

        FULL_STATE.rcu(|state| {
            let mut state = state.cloned();
            state.complete_initialization(&RESOURCES);
            state
        });

        Ok(())
    }

    pub async fn start() -> Result<()> {
        let state = FULL_STATE.load();
        rust_i18n::set_locale(state.locale());

        if state.is_weg_enabled() {
            SeelenWeg::hide_native_taskbar();
        }

        WIDGET_MANAGER.reconcile()?;
        create_background_window()?;
        register_win_hook()?;

        std::thread::spawn(init_clipboard_manager);

        start_discord_rpc()?;

        initialize_user_resources_watcher()?;

        let widgets = RESOURCES.widgets();
        let widget_refs: Vec<_> = widgets.iter().map(|w| w.as_ref()).collect();
        let (resolved, _) = resolve_shortcuts(&state.settings, &widget_refs);

        if !crate::cli::shortcuts::SHORTCUTS_PAUSED.load(std::sync::atomic::Ordering::Acquire) {
            ServicePipe::request(SvcAction::SetShortcuts(resolved))?;
        }

        Ok(())
    }

    pub fn on_settings_change(state: &AppSettings) -> Result<()> {
        update_discord_rpc(state.settings.drpc).log_error();
        rust_i18n::set_locale(state.locale());

        let widgets = RESOURCES.widgets();
        let widget_refs: Vec<_> = widgets.iter().map(|w| w.as_ref()).collect();
        let (resolved, has_conflicts) = resolve_shortcuts(&state.settings, &widget_refs);
        if has_conflicts {
            show_shortcut_conflict_popup().log_error();
        }
        if !crate::cli::shortcuts::SHORTCUTS_PAUSED.load(std::sync::atomic::Ordering::Acquire) {
            ServicePipe::request(SvcAction::SetShortcuts(resolved))?;
        }

        if state.is_weg_enabled() {
            SeelenWeg::hide_native_taskbar();
        } else {
            SeelenWeg::restore_native_taskbar()?;
        }

        // Re-emit user and session so streaming mode redaction takes effect immediately.
        reemit_user();
        reemit_session();
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
