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
    collections::HashMap,
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, LazyLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use itertools::Itertools;
use parking_lot::Mutex;
use windows::{
    core::GUID,
    Win32::UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT},
};

use crate::error::Result;

/// Writes `content` to `path` atomically: writes to a sibling `.tmp` file first,
/// syncs to disk, then renames into place. This guarantees the target file is
/// never left empty or partially written, even if the process is killed mid-write.
pub fn atomic_write_file(path: &Path, content: &[u8]) -> Result<()> {
    let dir = path.parent().ok_or("Path has no parent directory")?;
    create_dir_all(dir)?;

    let tmp_path = path.with_extension("tmp");
    let mut file = File::create(&tmp_path)?;
    file.write_all(content)?;
    file.flush()?;
    file.sync_all()?;
    drop(file); // must close before rename on Windows
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn pcwstr(s: &str) -> windows::core::PCWSTR {
    windows::core::PCWSTR::from_raw(s.encode_utf16().chain(Some(0)).collect_vec().as_ptr())
}

pub fn sleep_millis(millis: u64) {
    std::thread::sleep(Duration::from_millis(millis));
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
            let string_path =
                unsafe { SHGetKnownFolderPath(&rfid as _, KF_FLAG_DEFAULT, None)?.to_string()? };

            path_buf.push(string_path);
        } else if idx == 0 {
            return Ok(PathBuf::from(path.as_ref()));
        } else {
            path_buf.push(part);
        }
    }

    Ok(path_buf)
}

pub static TRACE_LOCK_ENABLED: AtomicBool = AtomicBool::new(true);
pub static LAST_SUCCESSFUL_LOCK: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[macro_export]
macro_rules! trace_lock {
    ($mutex:expr) => {
        trace_lock!($mutex, 5)
    };
    ($mutex:expr, $duration:expr) => {{
        let guard = $mutex.try_lock_for(std::time::Duration::from_secs($duration));
        let guard_name = stringify!($mutex);
        match guard {
            Some(guard) => {
                if $crate::utils::TRACE_LOCK_ENABLED.load(std::sync::atomic::Ordering::Acquire) {
                    let mut map = $crate::utils::LAST_SUCCESSFUL_LOCK
                        .try_lock_for(std::time::Duration::from_secs(5))
                        .unwrap();
                    let location = format!("{}:{}", file!(), line!());
                    map.insert(guard_name.to_owned(), location);
                }
                guard
            }
            None => {
                let mut panic_msg = format!(
                    "{} mutex is deadlocked at {}:{}",
                    guard_name,
                    file!(),
                    line!()
                );

                if let Some(path) = $crate::utils::LAST_SUCCESSFUL_LOCK
                    .try_lock_for(std::time::Duration::from_secs(5))
                    .unwrap()
                    .get(guard_name)
                {
                    panic_msg = format!("{}, last successful aquire was at {}", panic_msg, path);
                }

                panic!("{:?}", $crate::error::AppError::from(panic_msg));
            }
        }
    }};
}

pub static PERFORMANCE_HELPER: LazyLock<Mutex<PerformanceHelper>> = LazyLock::new(|| {
    Mutex::new(PerformanceHelper {
        time: HashMap::new(),
    })
});

pub struct PerformanceHelper {
    time: HashMap<String, Instant>,
}

impl PerformanceHelper {
    pub fn start(&mut self, name: &str) {
        log::debug!("{name} start");
        self.time.insert(name.to_string(), Instant::now());
    }

    pub fn elapsed(&self, name: &str) -> Duration {
        self.time.get(name).unwrap().elapsed()
    }

    pub fn end(&mut self, name: &str) {
        log::debug!("{} end in: {:.2}s", name, self.elapsed(name).as_secs_f64());
        self.time.remove(name);
    }
}

/// Useful when spawning threads that will allocate a loop or some other blocking operation
pub fn spawn_named_thread<F, T>(id: &str, cb: F) -> std::thread::JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    let thread = std::thread::Builder::new()
        .name(format!("Seelen Thread - {id}"))
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
