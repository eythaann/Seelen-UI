use crate::error::Result;

/// OS contract of the focus assist (do not disturb) module.
pub trait FocusAssistSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Acquires the OS resources and starts listening changes, reported through `on_event`.
    /// Called once; the subscription lives as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(FocusAssistEvent) + Send + Sync + 'static;

    /// Whether focus assist is currently active.
    fn is_active(&self) -> Result<bool>;

    /// Requests to turn focus assist on/off. OSes that don't allow to change it
    /// programmatically may open the related system settings instead.
    fn set_active(&self, enabled: bool) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusAssistEvent {
    ActiveChanged { active: bool },
}
