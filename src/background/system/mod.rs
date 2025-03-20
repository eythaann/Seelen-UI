pub mod brightness;

use tauri::Listener;

use crate::{
    error_handler::Result,
    log_error,
    modules::{
        bluetooth::{infrastructure::register_bluetooth_events, release_bluetooth_events},
        language::register_language_events,
        media::infrastructure::{register_media_events, release_media_events},
        monitors::infrastructure::register_monitor_webview_events,
        network::infrastructure::register_network_events,
        notifications::infrastructure::{
            register_notification_events, release_notification_events,
        },
        power::infrastructure::{register_power_events, release_power_events},
        shared::radio::RADIO_MANAGER,
        system_settings::infrastructure::{register_colors_events, release_colors_events},
        tray::infrastructure::register_tray_icons_events,
        user::infrastructure::register_user_events,
    },
    seelen::get_app_handle,
    trace_lock,
};

pub fn declare_system_events_handlers() -> Result<()> {
    // avoid binding interfaces to main thread
    std::thread::spawn(move || {
        log_error!(trace_lock!(RADIO_MANAGER).initialize());

        // todo change this to current implementation pattern
        let handle = get_app_handle();
        handle.listen("register-network-events", move |_| {
            log_error!(register_network_events());
        });

        register_tray_icons_events();
        register_notification_events();
        register_media_events();
        register_user_events();
        register_bluetooth_events();
        register_monitor_webview_events();
        register_colors_events();
        register_power_events();
        register_language_events();
    })
    .join()
    .expect("Failed to register system events");
    Ok(())
}

pub fn release_system_events_handlers() {
    release_notification_events();
    release_media_events();
    release_power_events();
    release_bluetooth_events();
    release_colors_events();
}
