//! Anonymous id of this installation, used by the telemetry.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use uuid::Uuid;

use crate::error::Result;

/// Returns the persisted install id, creating it if absent. It has to survive reinstalls and
/// updates of the app, so each OS stores it outside of the app data.
pub fn get_or_create() -> Result<Uuid> {
    imp::get_or_create()
}
