use seelen_core::{handlers::SeelenEvent, system_state::User};

use crate::{app::emit_to_webviews, state::application::FULL_STATE};

use super::application::UserManager;

fn maybe_redact_user(mut user: User) -> User {
    if FULL_STATE.load().settings.streaming_mode {
        user.email = Some("***@seelen.io".to_string());
    }
    user
}

pub fn reemit_user() {
    let user = UserManager::instance().lock().user.clone();
    emit_to_webviews(SeelenEvent::UserChanged, maybe_redact_user(user));
}

#[tauri::command(async)]
pub fn get_user() -> User {
    maybe_redact_user(UserManager::instance().lock().user.clone())
}
