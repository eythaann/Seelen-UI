use tauri::Listener;

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    modules::{
        apps::infrastructure::register_app_win_events,
        language::register_language_events,
        network::infrastructure::register_network_events,
        system_settings::infrastructure::{register_system_settings_events, release_colors_events},
        user::infrastructure::register_user_events,
    },
};

// todo replace this by self module lazy initilization
pub fn declare_system_events_handlers() -> Result<()> {
    // todo change this to current implementation pattern
    let handle = get_app_handle();
    handle.listen("register-network-events", move |_| {
        log_error!(register_network_events());
    });

    register_app_win_events();
    register_user_events();
    register_system_settings_events();
    register_language_events();
    // power events are registered lazily on first access
    Ok(())
}

pub fn release_system_events_handlers() {
    release_colors_events();
    // power events are released automatically on drop
}
