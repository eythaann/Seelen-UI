use crate::error::Result;

/// OS contract of the lifecycle module.
pub trait LifecycleSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening the OS lifecycle notifications, reported through `on_event`.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(LifecycleEvent) + Send + Sync + 'static;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent {
    /// The system is going to sleep/hibernate.
    Suspending,
    /// The system woke up.
    Resumed,
    /// The OS wants the process to exit: shutdown, logoff, or a package update needing it.
    EndSession,
}
