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
        power::infrastructure::PowerManager,
        system_settings::infrastructure::{register_colors_events, release_colors_events},
        tray::infrastructure::register_tray_events,
    },
    seelen::get_app_handle,
};

pub fn declare_system_events_handlers() -> Result<()> {
    let handle = get_app_handle();

    handle.listen("register-power-events", move |_| {
        log_error!(PowerManager::register_power_events());
        log_error!(PowerManager::emit_system_power_info());
    });

    handle.listen("register-tray-events", move |_| register_tray_events());

    handle.listen("register-network-events", move |_| {
        log_error!(register_network_events());
    });

    handle.listen("register-bluetooth-events", move |_| {
        // todo
    });

    handle.listen("register-media-events", move |_| {
        register_media_events();
    });

    handle.listen("register-notifications-events", move |_| {
        register_notification_events();
    });

    register_monitor_webview_events()?;
    register_colors_events();
    Ok(())
}

pub fn release_system_events_handlers() {
    release_media_events();
    release_notification_events();
    release_colors_events();
}
