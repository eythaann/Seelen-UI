use std::path::Path;

use crate::{error::Result, utils::constants::SEELEN_COMMON, windows_api::WindowsApi};

/// `ShellExecute` inside explorer.exe, falling back to our own process.
pub fn execute(
    program: &str,
    args: Option<&str>,
    working_dir: Option<&Path>,
    elevated: bool,
) -> Result<()> {
    WindowsApi::execute(
        program.to_owned(),
        args.map(str::to_owned),
        working_dir.map(Path::to_path_buf),
        elevated,
    )
}

pub fn reveal(path: &Path) -> Result<()> {
    std::process::Command::new(SEELEN_COMMON.system_dir().join("explorer.exe"))
        .arg("/select,")
        .arg(path)
        .spawn()?;
    Ok(())
}
