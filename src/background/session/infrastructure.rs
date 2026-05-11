use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::SeelenSession};

use crate::{
    app::emit_to_webviews,
    error::{Result, ResultLogExt},
    resources::RESOURCES,
    state::application::FULL_STATE,
};

use super::application::{SessionManager, SessionManagerEvent};

fn maybe_redact_session(session: Option<SeelenSession>) -> Option<SeelenSession> {
    session.map(|mut s| {
        if FULL_STATE.load().settings.streaming_mode {
            s.email = "***@seelen.io".to_string();
        }
        s
    })
}

fn get_session_manager() -> &'static parking_lot::Mutex<SessionManager> {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        SessionManager::subscribe(|event| {
            let SessionManagerEvent::Changed(session) = event;
            emit_to_webviews(
                SeelenEvent::SeelenSessionChanged,
                maybe_redact_session(session),
            );
            RESOURCES.emit_on_session_changed().log_error();
        });
    });
    SessionManager::instance()
}

pub fn reemit_session() {
    let session = get_session_manager().lock().session.clone();
    emit_to_webviews(
        SeelenEvent::SeelenSessionChanged,
        maybe_redact_session(session),
    );
}

/// Returns the current session (user data without tokens) or `null` if not
/// authenticated. Safe to call from any widget.
#[tauri::command(async)]
pub fn get_seelen_session() -> Option<SeelenSession> {
    maybe_redact_session(get_session_manager().lock().session.clone())
}

/// Opens the system browser to the Seelen sign-in page. After the user
/// authenticates, the website redirects back via the deep-link URI scheme and
/// the session is established automatically. Tokens are stored exclusively in
/// the Windows Credential Manager and never returned to the UI.
#[tauri::command(async)]
pub fn seelen_login() -> Result<()> {
    get_session_manager(); // ensure event subscription is initialised
    SessionManager::login()
}

/// Invalidates the current session server-side and wipes all locally stored
/// tokens from the Windows Credential Manager.
#[tauri::command(async)]
pub async fn seelen_logout() -> Result<()> {
    get_session_manager(); // ensure event subscription is initialised
    SessionManager::logout().await
}
