#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::{RadioDevice, RadioDeviceKind};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    system_modules::{bluetooth::BluetoothManager, wifi::WifiManager},
};

use super::domain::{RadiosEvent, RadiosSysEvent, RadiosSystem};

#[cfg(windows)]
type Platform = windows::WindowsRadios;

/// Stateless: the radios are tied to their OS watchers, so the native layer keeps them.
pub struct RadioManager {
    platform: Platform,
}

event_manager!(RadioManager, RadiosEvent);

impl RadioManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<RadioManager> = LazyLock::new(|| {
            let mut m = RadioManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)
    }

    fn on_platform_event(event: RadiosSysEvent) {
        match event {
            RadiosSysEvent::Changed => Self::send(RadiosEvent::Changed),
            // Neither the WiFi network scan nor the Bluetooth discovery watchers reliably
            // resume on their own right after their radio is turned back on, so kick them.
            RadiosSysEvent::Enabled { kind } => match kind {
                RadioDeviceKind::WiFi => WifiManager::instance().scan(),
                RadioDeviceKind::Bluetooth => BluetoothManager::instance()
                    .restart_scanning_if_active()
                    .log_error(),
                _ => {}
            },
        }
    }

    pub fn radios(&self) -> Vec<RadioDevice> {
        self.platform.radios()
    }

    pub fn set_enabled(&self, kind: RadioDeviceKind, enabled: bool) -> Result<()> {
        self.platform.set_enabled(kind, enabled)
    }
}
