//! Launching programs, documents and URLs through the OS shell.

#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows as imp;

use std::path::Path;

use crate::error::Result;

/// Launches a program, document or URL with the OS shell, detached from our process (it doesn't
/// inherit our console, environment nor elevation).
pub fn execute(
    program: &str,
    args: Option<&str>,
    working_dir: Option<&Path>,
    elevated: bool,
) -> Result<()> {
    imp::execute(program, args, working_dir, elevated)
}

/// Opens a file or URL with its default handler.
pub fn open(target: &str) -> Result<()> {
    execute(target, None, None, false)
}

/// Shows the file selected in the OS file manager.
pub fn reveal(path: &Path) -> Result<()> {
    imp::reveal(path)
}
