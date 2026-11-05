use seelen_core::system_state::TrashBinInfo;

use crate::error::Result;

/// OS contract of the trash bin (recycle bin) module.
pub trait TrashBinSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening OS changes, reported through `on_event`. The OS may notify changes that
    /// leave the info untouched: deduplicating them is up to the caller.
    /// Called once; the subscription lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(TrashBinEvent) + Send + Sync + 'static;

    /// Current item count and size of the trash bin.
    fn query(&self) -> Result<TrashBinInfo>;

    /// Permanently deletes every item in the trash bin.
    fn empty(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum TrashBinEvent {
    InfoChanged { info: TrashBinInfo },
}
