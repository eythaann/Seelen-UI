use tauri::Listener;

use crate::{
    app::get_app_handle, error::Result, log_error,
    modules::network::infrastructure::register_network_events,
};

// todo replace this by self module lazy initilization
pub fn declare_system_events_handlers() -> Result<()> {
    // todo change this to current implementation pattern
    let handle = get_app_handle();
    handle.listen("register-network-events", move |_| {
        log_error!(register_network_events());
    });

    // power, system_settings, language, apps, and user events are registered lazily on first access
    Ok(())
}
