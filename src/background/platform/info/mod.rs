//! Information about the OS and the way Seelen UI is installed/running on it.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

#[cfg(windows)]
pub use self::windows::{get_fixed_runtime_path, is_windows_10};

/// The webview runtime is a fixed version bundled with the app instead of the one installed on
/// the system.
pub fn has_fixed_runtime() -> bool {
    imp::has_fixed_runtime()
}

/// Seelen UI is running inside an OS app package (Windows: APPX/MSIX).
pub fn is_running_as_appx() -> bool {
    imp::is_running_as_appx()
}

/// Seelen UI was installed through an OS app package (Windows: MSIX), even if this process was
/// started outside of it.
pub fn was_installed_using_msix() -> bool {
    imp::was_installed_using_msix()
}
