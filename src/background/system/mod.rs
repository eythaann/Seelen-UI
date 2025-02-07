pub mod brightness;

use tauri::Listener;

use crate::{
    error_handler::Result,
    log_error,
    modules::{
        media::infrastructure::{register_media_events, release_media_events},
        monitors::infrastructure::register_monitor_webview_events,
        network::infrastructure::register_network_events,
        notifications::infrastructure::{
            register_notification_events, release_notification_events,
        },
        power::infrastructure::{release_power_events, PowerManager},
        system_settings::infrastructure::{register_colors_events, release_colors_events},
        tray::infrastructure::register_tray_events,
        user::infrastructure::register_user_events,
    },
    seelen::get_app_handle,
};

pub fn declare_system_events_handlers() -> Result<()> {
    let handle = get_app_handle();

    // todo change this to current implementation pattern
    handle.listen("register-power-events", move |_| {
        log_error!(PowerManager::register_power_events());
        log_error!(PowerManager::emit_system_power_info());
    });

    // todo change this to current implementation pattern
    handle.listen("register-tray-events", move |_| register_tray_events());

    // todo change this to current implementation pattern
    handle.listen("register-network-events", move |_| {
        log_error!(register_network_events());
    });

    // todo change this to current implementation pattern
    handle.listen("register-media-events", move |_| {
        register_media_events();
    });

    // todo change this to current implementation pattern
    handle.listen("register-notifications-events", move |_| {
        register_notification_events();
    });

    register_user_events();
    register_monitor_webview_events();
    register_colors_events();
    Ok(())
}

pub fn release_system_events_handlers() {
    release_media_events();
    release_power_events();
    release_notification_events();
    release_colors_events();
}
