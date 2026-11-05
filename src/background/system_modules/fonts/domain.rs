use seelen_core::system_state::SeelenFont;

use crate::error::Result;

/// OS contract of the fonts module.
pub trait FontsSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening OS changes, reported through `on_event`.
    /// Called once; the subscription lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(FontsSysEvent) + Send + Sync + 'static;

    /// Font families installed on the system, in no particular order.
    fn fonts(&self) -> Result<Vec<SeelenFont>>;
}

/// Raw OS notifications: they only signal a change, the manager queries the new list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontsSysEvent {
    /// A font was installed or removed.
    Changed,
}

#[derive(Debug, Clone)]
pub enum FontsEvent {
    Changed { fonts: Vec<SeelenFont> },
}
