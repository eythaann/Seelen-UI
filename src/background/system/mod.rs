pub mod brightness;

use crate::{
    error_handler::Result, modules::power::infrastructure::PowerManager, seelen::get_app_handle,
};

pub fn register_system_events() -> Result<()> {
    let handle = get_app_handle();

    handle.once("register-power-events", move |_| {
        PowerManager::register_power_events().expect("Fail on registering system power events");
        PowerManager::emit_system_power_info().expect("Fail on emitting initial system power info");
    });

    handle.once("register-wifi-events", move |_| {
        // todo
    });

    handle.once("register-bluetooth-events", move |_| {
        // todo
    });

    handle.once("register-audio-events", move |_| {
        // todo
        // let audio = WindowsApi::get_default_audio_endpoint().unwrap();
        // audio.RegisterControlChangeNotify(IAudioEndpointVolumeCallback)
    });

    Ok(())
}
