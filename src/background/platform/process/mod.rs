#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::path::PathBuf;

use seelen_core::system_state::ProcessInformation;

use crate::error::Result;

#[cfg(windows)]
pub use self::windows::WindowsProcessExt;

/// A running process of any application.
pub struct Process(imp::NativeProcess);

impl Process {
    pub fn from_id(id: u32) -> Self {
        Self(imp::NativeProcess::from_id(id))
    }

    pub fn id(&self) -> u32 {
        self.0.id()
    }

    /// Path of the executable that started the process.
    pub fn exe_path(&self) -> Result<PathBuf> {
        self.0.exe_path()
    }

    /// File name of [`Process::exe_path`].
    pub fn exe_name(&self) -> Result<String> {
        Ok(self
            .exe_path()?
            .file_name()
            .ok_or("there is no file name")?
            .to_string_lossy()
            .to_string())
    }

    /// Human readable name of the program, falls back to the executable name.
    pub fn display_name(&self) -> Result<String> {
        self.0.display_name()
    }

    /// Suspended by the OS (e.g. UWP apps in background).
    pub fn is_frozen(&self) -> Result<bool> {
        self.0.is_frozen()
    }

    pub fn is_seelen(&self) -> bool {
        if let Ok(exe) = self.exe_path() {
            return exe.ends_with("seelen-ui.exe");
        }
        false
    }

    pub fn snapshot(&self) -> ProcessInformation {
        ProcessInformation {
            id: self.id(),
            path: self.exe_path().ok(),
        }
    }
}
