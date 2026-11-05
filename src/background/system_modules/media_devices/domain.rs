use seelen_core::system_state::MediaDevice;

use crate::error::Result;

/// OS contract of the audio devices module.
pub trait MediaDevicesSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Loads the devices and starts listening their changes, reported through `on_event`.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(MediaDevicesSysEvent) + Send + Sync + 'static;

    /// Capture devices (microphones), with their sessions.
    fn inputs(&self) -> Vec<MediaDevice>;

    /// Render devices (speakers, headphones), with their sessions.
    fn outputs(&self) -> Vec<MediaDevice>;

    /// `role` is one of `multimedia`, `communications` or `console`.
    fn set_default_device(&self, device_id: &str, role: &str) -> Result<()>;

    /// Toggles the mute of the device, or of one of its sessions.
    fn toggle_mute(&self, device_id: &str, session_id: Option<&str>) -> Result<()>;

    /// Sets the volume (`0.0..=1.0`) of the device, or of one of its sessions.
    fn set_volume_level(&self, device_id: &str, session_id: Option<&str>, level: f32)
    -> Result<()>;
}

/// Raw OS notifications: they only signal a change, the manager queries the devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaDevicesSysEvent {
    /// Any device, session, volume or default device changed.
    Changed,
    /// The default multimedia output changed (also reported as `Changed`).
    DefaultOutputChanged,
}

#[derive(Debug, Clone)]
pub enum MediaDevicesEvent {
    Changed {
        inputs: Vec<MediaDevice>,
        outputs: Vec<MediaDevice>,
    },
    DefaultOutputChanged,
}
