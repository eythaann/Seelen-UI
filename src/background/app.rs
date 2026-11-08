use slu_ipc::messages::SvcAction;
use tauri::{AppHandle, Emitter, Wry};

use seelen_core::state::shortcuts::resolve_shortcuts;

use crate::{
    APP_HANDLE,
    cli::ServicePipe,
    error::{Result, ResultLogExt},
    migrations::Migrations,
    platform::{self, startup},
    resources::RESOURCES,
    session::reemit_session,
    state::{AppSettings, FULL_STATE, initialize_user_resources_watcher},
    system_modules::{
        clipboard::init_clipboard_manager, start::StartMenuManager, user::reemit_user,
    },
    utils::discord::{start_discord_rpc, update_discord_rpc},
    widgets::{
        manager::WIDGET_MANAGER, popups::shortcut_conflicts::show_shortcut_conflict_popup,
        weg::SeelenWeg,
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
    S: serde::Serialize,
{
    if !crate::platform::session::is_interactive() {
        return;
    }

    // serialized once and shared by both transports
    let payload = match serde_json::to_string(&payload) {
        Ok(payload) => payload,
        Err(err) => {
            log::error!("Failed to serialize event {event}: {err}");
            return;
        }
    };

    crate::server::events::EVENTS_HUB.publish(event, &payload);
    // kept for backward compatibility with widgets listening via the tauri api
    get_app_handle().emit_str(event, payload).log_error();
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
        platform::start()?;

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
        startup::is_enabled()
    }

    pub fn set_auto_start(enabled: bool) -> Result<()> {
        ServicePipe::request(SvcAction::SetStartup(enabled))
    }
}
