//! Startup entry used to launch the Seelen UI service on user logon.
//!
//! Registering/unregistering the entry needs elevation, so it is done by the service itself
//! (`SvcAction::SetStartup`); the app only reads it and uses it to launch the service.

#[cfg(windows)]
mod windows;

use std::path::Path;

use crate::error::Result;

#[cfg(windows)]
type Native = windows::WindowsStartup;

/// Contract to be implemented by each OS.
trait StartupApi {
    fn is_enabled() -> Result<bool>;
    fn launch_service(service_exe: &Path) -> Result<()>;
}

/// Whether the service is configured to start on user logon.
pub fn is_enabled() -> Result<bool> {
    Native::is_enabled()
}

/// Launches the service through its startup entry, so it starts with the privileges it was
/// registered with. Fails if the entry doesn't exist or points to another executable than
/// `service_exe` (e.g. another installation).
pub fn launch_service(service_exe: &Path) -> Result<()> {
    Native::launch_service(service_exe)
}
