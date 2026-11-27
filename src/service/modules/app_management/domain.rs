use crate::error::Result;

/// OS contract of the app management module.
pub trait AppManagementSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here.
    fn create() -> Self;

    /// Launches Seelen UI on the interactive session, with the user's (non elevated) token.
    fn launch_app(&self) -> Result<()>;

    /// Kills every Seelen UI process of the current OS user session.
    fn kill_app_instances(&self) -> Result<()>;
}
