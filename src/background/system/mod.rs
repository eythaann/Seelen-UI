pub mod brightness;

use tauri::Listener;

use crate::{
    error_handler::Result,
    modules::{
        media::infrastructure::{register_media_events, release_media_events},
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

    handle.once("register-power-events", move |_| {
        log::debug!("Registering system power events");
        PowerManager::register_power_events().expect("Fail on registering system power events");
        PowerManager::emit_system_power_info().expect("Fail on emitting initial system power info");
    });

    handle.once("register-tray-events", move |_| {
        log::debug!("Registering tray events");
        register_tray_events().expect("Fail on registering tray events");
    });

    handle.once("register-network-events", move |_| {
        log::debug!("Registering network events");
        register_network_events().expect("Fail on registering network events");
    });

    handle.once("register-bluetooth-events", move |_| {
        // todo
    });

    handle.listen("register-media-events", move |_| {
        log::debug!("Registering media events");
        register_media_events();
    });

    handle.listen("register-notifications-events", move |_| {
        log::debug!("Registering notifications events");
        register_notification_events();
    });

    handle.listen("register-colors-events", move |_| {
        log::debug!("Registering colors events");
        register_colors_events();
    });

    Ok(())
}

pub fn release_system_events_handlers() {
    release_media_events();
    release_notification_events();
    release_colors_events();
}
