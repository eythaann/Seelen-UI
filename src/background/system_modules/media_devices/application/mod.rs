#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::MediaDevice;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{MediaDevicesEvent, MediaDevicesSysEvent, MediaDevicesSystem};

#[cfg(windows)]
type Platform = windows::WindowsMediaDevices;

/// Stateless: the devices are tied to their OS handles, so the native layer keeps them.
pub struct MediaDevicesManager {
    platform: Platform,
}

event_manager!(MediaDevicesManager, MediaDevicesEvent);

impl MediaDevicesManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<MediaDevicesManager> = LazyLock::new(|| {
            let mut m = MediaDevicesManager::create();
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

    fn on_platform_event(event: MediaDevicesSysEvent) {
        if event == MediaDevicesSysEvent::DefaultOutputChanged {
            Self::send(MediaDevicesEvent::DefaultOutputChanged);
        }
        let manager = Self::instance();
        Self::send(MediaDevicesEvent::Changed {
            inputs: manager.inputs(),
            outputs: manager.outputs(),
        });
    }

    pub fn inputs(&self) -> Vec<MediaDevice> {
        self.platform.inputs()
    }

    pub fn outputs(&self) -> Vec<MediaDevice> {
        self.platform.outputs()
    }

    pub fn set_default_device(&self, device_id: &str, role: &str) -> Result<()> {
        self.platform.set_default_device(device_id, role)
    }

    pub fn toggle_mute(&self, device_id: &str, session_id: Option<&str>) -> Result<()> {
        self.platform.toggle_mute(device_id, session_id)
    }

    pub fn set_volume_level(
        &self,
        device_id: &str,
        session_id: Option<&str>,
        level: f32,
    ) -> Result<()> {
        self.platform
            .set_volume_level(device_id, session_id, level.clamp(0.0, 1.0))
    }
}
