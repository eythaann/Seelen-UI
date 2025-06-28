pub mod ahk;
pub mod constants;
pub mod discord;
pub mod icon_extractor;
pub mod integrity;
pub mod pwsh;
pub mod updater;
pub mod virtual_desktop;
mod winver;

use base64::Engine;
use uuid::Uuid;
pub use winver::*;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::{
    core::GUID,
    Win32::{
        Foundation::RECT,
        UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT},
    },
};

use crate::error_handler::Result;

pub fn pcwstr(s: &str) -> windows::core::PCWSTR {
    windows::core::PCWSTR::from_raw(s.encode_utf16().chain(Some(0)).collect_vec().as_ptr())
}

pub fn sleep_millis(millis: u64) {
    std::thread::sleep(Duration::from_millis(millis));
}

pub fn are_overlaped(a: &RECT, b: &RECT) -> bool {
    let zeroed = RECT::default();
    if a == &zeroed || b == &zeroed {
        return false;
    }
    // The edge pixel overlapping do not matters. This resolves the shared pixel in between the monitors,
    // hereby a fullscreened app shared pixel collision does not hide other monitor windows.
    if a.right <= b.left || a.left >= b.right || a.bottom <= b.top || a.top >= b.bottom {
        return false;
    }
    true
}

/// Resolve paths with folder ids in the form of "{GUID}\path\to\file"
///
/// https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
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

pub static TRACE_LOCK_ENABLED: AtomicBool = AtomicBool::new(false);
lazy_static::lazy_static! {
    pub static ref LAST_SUCCESSFUL_LOCK: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

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
                    log::trace!("{} lock acquired at {}", guard_name, location);
                    map.insert(guard_name.to_owned(), location);
                }
                guard
            }
            None => {
                let mut panic_msg = format!("{} mutex is deadlocked", guard_name);
                if let Some(path) = $crate::utils::LAST_SUCCESSFUL_LOCK
                    .try_lock_for(std::time::Duration::from_secs(5))
                    .unwrap()
                    .get(guard_name)
                {
                    panic_msg = format!("{}, last successful aquire was at {}", panic_msg, path);
                }
                panic!("{:?}", $crate::error_handler::AppError::from(panic_msg));
            }
        }
    }};
}

lazy_static! {
    pub static ref PERFORMANCE_HELPER: Mutex<PerformanceHelper> = Mutex::new(PerformanceHelper {
        time: HashMap::new(),
    });
}

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
pub fn spawn_named_thread<F, T>(id: &str, cb: F) -> Result<std::thread::JoinHandle<T>>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    std::thread::Builder::new()
        .name(format!("Seelen Thread - {id}"))
        .spawn(cb)
        .map_err(|e| format!("Failed to spawn thread: {e}").into())
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
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

pub struct WidgetWebviewLabel {
    /// this should be used as the real webview label
    pub raw: String,
    /// this is the decoded label, useful for debugging and logging
    pub decoded: String,
}

impl WidgetWebviewLabel {
    pub fn new(widget_id: &str, monitor_id: Option<&str>, instance_id: Option<&Uuid>) -> Self {
        let mut label = widget_id.to_string();
        let with_monitor_id = monitor_id.is_some();
        let with_instance_id = instance_id.is_some();
        if with_monitor_id || with_instance_id {
            label.push('?');
        }

        if let Some(monitor_id) = monitor_id {
            label.push_str(&format!("monitorId={}", urlencoding::encode(monitor_id)));
        }

        if let Some(instance_id) = instance_id {
            if with_monitor_id {
                label.push('&');
            }
            label.push_str(&format!(
                "instanceId={}",
                urlencoding::encode(&instance_id.to_string())
            ));
        }

        Self {
            raw: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&label),
            decoded: label,
        }
    }
}

#[allow(dead_code)]
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
