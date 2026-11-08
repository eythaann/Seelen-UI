mod application;
mod domain;

use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::User};

use crate::{app::emit_to_webviews, state::FULL_STATE};

use self::{application::UserManager, domain::UserEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static UserManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        UserManager::subscribe(|event| match event {
            UserEvent::Changed { user } => {
                emit_to_webviews(SeelenEvent::UserChanged, redact_user(user));
            }
        });
    });
    UserManager::instance()
}

/// Streaming mode hides the personal data from the webviews.
fn redact_user(mut user: User) -> User {
    if FULL_STATE.load().settings.streaming_mode {
        user.email = Some("***@seelen.io".to_string());
    }
    user
}

/// Emits the user again, as the redaction depends on the settings.
pub fn reemit_user() {
    emit_to_webviews(SeelenEvent::UserChanged, redact_user(get_manager().user()));
}

impl crate::tauri_handlers::Handlers {
    pub fn get_user() -> User {
        redact_user(get_manager().refresh())
    }
}
