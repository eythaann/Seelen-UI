pub mod brightness;

use crate::{
    error_handler::Result,
    modules::{
        media::infrastructure::register_media_events,
        network::infrastructure::register_network_events, power::infrastructure::PowerManager,
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

    Ok(())
}
