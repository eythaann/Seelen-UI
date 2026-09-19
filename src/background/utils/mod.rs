pub mod constants;
pub mod discord;
pub mod icon_extractor;
pub mod integrity;
pub mod lock_free;
pub mod pwsh;
pub mod updater;
pub mod virtual_desktop;
mod winver;

pub use winver::*;

use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use windows::{
    Win32::UI::Shell::{KF_FLAG_DEFAULT, SHGetKnownFolderPath},
    core::GUID,
};

use crate::{error::Result, windows_api::string_utils::WindowsString};

static ATOMIC_WRITE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Writes `content` to `path` atomically: writes to a sibling temporary file first,
/// syncs to disk, then renames into place. This guarantees the target file is
/// never left empty or partially written, even if the process is killed mid-write.
///
/// The temporary file name is unique per call. Two concurrent writers of the same
/// target (e.g. one dock webview per monitor saving its state at the same time)
/// used to share a single `.tmp` file: the first rename moved it away and the
/// second one failed with `NotFound`.
pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<()> {
    let dir = path.parent().ok_or("Path has no parent directory")?;
    create_dir_all(dir)?;

    let file_name = path
        .file_name()
        .ok_or("Path has no file name")?
        .to_string_lossy();
    let tmp_path = path.with_file_name(format!(
        "{file_name}.{}.{}.tmp",
        std::process::id(),
        ATOMIC_WRITE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));

    let mut file = File::create(&tmp_path)?;
    file.write_all(content)?;
    file.flush()?;
    file.sync_all()?;
    drop(file); // must close before rename on Windows
    if let Err(err) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(err.into());
    }
    Ok(())
}

/// Resolve paths with folder ids in the form of "{GUID}\path\to\file"
///
/// https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
#[allow(dead_code)]
pub fn resolve_guid_path<S: AsRef<str>>(path: S) -> Result<PathBuf> {
    let parts = path.as_ref().split("\\");
    let mut path_buf = PathBuf::new();

    for (idx, part) in parts.into_iter().enumerate() {
        if part.starts_with("{") && part.ends_with("}") {
            let guid = part.trim_start_matches('{').trim_end_matches('}');
            let rfid = GUID::try_from(guid)?;
            let string_path = WindowsString::from(unsafe {
                SHGetKnownFolderPath(&rfid as _, KF_FLAG_DEFAULT, None)?
            })
            .to_string();

            path_buf.push(string_path);
        } else if idx == 0 {
            return Ok(PathBuf::from(path.as_ref()));
        } else {
            path_buf.push(part);
        }
    }

    Ok(path_buf)
}

/// Useful when spawning threads that will allocate a loop or some other blocking operation
pub fn spawn_named_thread<F, T>(id: &str, cb: F) -> std::thread::JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let thread = std::thread::Builder::new()
        .name(format!("SLU - {id}"))
        .spawn(cb);
    match thread {
        Ok(handle) => handle,
        Err(e) => panic!("Failed to spawn thread: {e}"),
    }
}

/// intended to work as converFileToSrc in JS side using tauri library
pub fn convert_file_to_src(path: &Path) -> String {
    #[cfg(any(windows, target_os = "android"))]
    let base = "http://asset.localhost/";
    #[cfg(not(any(windows, target_os = "android")))]
    let base = "asset://localhost/";
    let path = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();
    let encoded = urlencoding::encode(&path);
    format!("{base}{encoded}")
}

pub fn now_timestamp_as_millis() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000
}

pub fn date_based_hex_id() -> String {
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{since_epoch:x}")
}

pub fn get_parts_of_inline_command(cmd: &str) -> (String, Option<String>) {
    let start_double_quoted = cmd.starts_with("\"");
    if start_double_quoted || cmd.starts_with("'") {
        let delimiter = if start_double_quoted { '"' } else { '\'' };
        let mut parts = cmd.split(['"', '\'']).filter(|s| !s.is_empty());

        let program = parts.next().unwrap_or_default().trim().to_owned();
        let args = cmd
            .trim_start_matches(&format!("{delimiter}{program}{delimiter}"))
            .trim()
            .to_owned();
        return (program, if args.is_empty() { None } else { Some(args) });
    }

    let cmd_as_path = PathBuf::from(cmd);
    if cmd_as_path.exists() {
        let program = cmd_as_path.to_string_lossy().to_string();
        return (program, None);
    }

    let mut parts = cmd.split(" ").filter(|s| !s.is_empty());
    let program = parts.next().unwrap_or_default().trim().to_owned();
    let args = cmd.trim_start_matches(&program).trim().to_owned();
    (program, if args.is_empty() { None } else { Some(args) })
}

#[macro_export]
macro_rules! measure {
    ($name:literal, $expr:expr) => {{
        let start = std::time::Instant::now();
        let result = $expr;
        let elapsed = start.elapsed();
        log::debug!("[measure] {}: {:.2?}", $name, elapsed);
        result
    }};
}

pub fn collect_files(dir: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect()
}
