//! OS primitives of the service process itself (not owned by any module).
//!
//! The public API is the contract: it only forwards to the native module aliased as `imp`, so a
//! new OS has to provide the same free functions and the compiler lists whatever is missing.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::path::PathBuf;

use crate::error::Result;

/// Prepares the process to run as the service: DPI awareness and the privileges it needs.
pub fn prepare_process() -> Result<()> {
    imp::prepare_process()
}

/// Whether another instance of the service is already running in the current OS user session.
/// When it is not, this process claims the instance until it exits.
pub fn is_already_running() -> bool {
    imp::is_already_running()
}

/// Blocks until the native shell is ready (5 seconds max), so the app is not launched before it.
pub fn wait_for_native_shell() {
    imp::wait_for_native_shell()
}

/// Minimizes the console window attached to the process, if any (local development).
pub fn minimize_console() {
    imp::minimize_console()
}

/// Per user local data folder of Seelen UI (logs, saved state).
pub fn local_data_dir() -> Result<PathBuf> {
    Ok(imp::local_app_data()?.join("com.seelen.seelen-ui"))
}
