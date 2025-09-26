use tauri::Listener;

use crate::{
    app::get_app_handle,
    error::Result,
    log_error,
    modules::{
        apps::infrastructure::register_app_win_events,
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
        system_settings::infrastructure::{register_system_settings_events, release_colors_events},
        tray::infrastructure::register_tray_icons_events,
        user::infrastructure::register_user_events,
    },
    trace_lock,
};

// todo replace this by self module lazy initilization
pub fn declare_system_events_handlers() -> Result<()> {
    // avoid binding interfaces to main thread
    // others like bluetooth or wi-fi, bandwidth, etc depends on this.
    std::thread::spawn(move || {
        log_error!(trace_lock!(RADIO_MANAGER).initialize());
    })
    .join()
    .expect("Failed to register system events");

    // todo change this to current implementation pattern
    let handle = get_app_handle();
    handle.listen("register-network-events", move |_| {
        log_error!(register_network_events());
    });

    register_app_win_events();
    register_tray_icons_events();
    register_notification_events();
    register_media_events();
    register_user_events();
    register_bluetooth_events();
    log_error!(register_monitor_webview_events());
    register_system_settings_events();
    register_power_events();
    register_language_events();
    Ok(())
}

pub fn release_system_events_handlers() {
    release_notification_events();
    release_media_events();
    release_power_events();
    release_bluetooth_events();
    release_colors_events();
}
