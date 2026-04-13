use std::sync::Once;

use crate::session::application::{SessionManager, SessionManagerEvent};

use super::application::run_cloud_sync;

fn register_session_listener() {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        SessionManager::subscribe(|event| {
            let SessionManagerEvent::Changed(session) = event;
            if session.is_some() {
                crate::get_tokio_handle().spawn(async {
                    run_cloud_sync().await;
                });
            }
        });
    });
}

/// Called once at app startup. Registers the session-change hook and fires
/// an initial reconciliation check.
pub fn start_backup_sync() {
    register_session_listener();
    crate::get_tokio_handle().spawn(async {
        run_cloud_sync().await;
    });
}
