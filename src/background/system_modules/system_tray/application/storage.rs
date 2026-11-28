use std::{
    io::{BufReader, Write},
    path::PathBuf,
};

use crate::{error::Result, utils::constants::SEELEN_COMMON};

use super::super::domain::TrayIconConfig;

const FILE_NAME: &str = "tray_icons.json";

/// Reads the stored icon settings, `None` if they were never stored.
pub fn load() -> Result<Option<Vec<TrayIconConfig>>> {
    let path = file_path();
    if !path.exists() {
        return Ok(None);
    }
    let file = std::fs::File::open(path)?;
    Ok(Some(serde_json::from_reader(BufReader::new(file))?))
}

pub fn save(configs: &[TrayIconConfig]) -> Result<()> {
    let mut file = std::fs::File::create(file_path())?;
    serde_json::to_writer(&mut file, configs)?;
    file.flush()?;
    log::trace!("Tray icons settings saved");
    Ok(())
}

fn file_path() -> PathBuf {
    SEELEN_COMMON.app_cache_dir().join(FILE_NAME)
}
