pub mod brightness;
pub mod power;

use crate::{error_handler::Result, seelen::get_app_handle};

pub fn register_system_events() -> Result<()> {
    let handle = get_app_handle();

    handle.once("register-power-events", move |_| {
        power::register_battery_events();
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
