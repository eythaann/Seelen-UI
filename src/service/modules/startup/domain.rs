use crate::error::Result;

/// OS contract of the startup module.
pub trait StartupSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Creates or repairs the entry that starts the service (with `--startup`) on logon,
    /// keeping the run on logon choice of a previous entry.
    fn register_service(&self) -> Result<()>;

    /// Enables/disables starting the service on logon. The entry must exist.
    fn set_run_on_logon(&self, enabled: bool) -> Result<()>;

    /// Removes the startup entry, no-op if it doesn't exist.
    fn unregister_service(&self) -> Result<()>;
}
