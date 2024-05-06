pub mod power;
pub mod brightness;

use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;
use tauri::{AppHandle, Wry};

use crate::error_handler::Result;

lazy_static! {
    pub static ref HANDLER: Arc<Mutex<Handler>> = Arc::new(Mutex::new(Handler::new()));
}

pub struct Handler(Option<AppHandle<Wry>>);
impl Handler {
    fn new() -> Self {
        Self(None)
    }

    fn init(&mut self, handle: AppHandle<Wry>) {
        self.0 = Some(handle);
    }

    fn clone_handle(&self) -> AppHandle<Wry> {
        self.0.clone().unwrap()
    }
}

pub fn register_system_events(handle: AppHandle<Wry>) -> Result<()> {
    HANDLER.lock().init(handle.clone());

    handle.once("register-power-events", move |_| {
        power::register_battery_events(HANDLER.lock().clone_handle());
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
