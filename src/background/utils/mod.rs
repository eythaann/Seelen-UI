pub mod ahk;
pub mod constants;
pub mod integrity;
pub mod pwsh;
pub mod updater;
pub mod virtual_desktop;
mod winver;

pub use winver::*;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
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

pub fn pascal_to_kebab(input: &str) -> String {
    let mut kebab_case = String::new();
    let mut prev_char_lowercase = false;
    for c in input.chars() {
        if c.is_uppercase() {
            if prev_char_lowercase {
                kebab_case.push('-');
            }
            kebab_case.push(c.to_ascii_lowercase());
            prev_char_lowercase = false;
        } else {
            kebab_case.push(c);
            prev_char_lowercase = true;
        }
    }
    kebab_case
}

pub fn kebab_to_pascal(input: &str) -> String {
    let mut pascal_case = String::new();
    let mut prev_char_dash = false;
    for c in input.chars() {
        if c == '-' {
            prev_char_dash = true;
        } else if prev_char_dash || pascal_case.is_empty() {
            pascal_case.push(c.to_ascii_uppercase());
            prev_char_dash = false;
        } else {
            pascal_case.push(c);
        }
    }
    pascal_case
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
                    let mut map = $crate::utils::LAST_SUCCESSFUL_LOCK.lock();
                    let location = format!("{}:{}", file!(), line!());
                    log::trace!("{} lock acquired at {}", guard_name, location);
                    map.insert(guard_name.to_owned(), location);
                }
                guard
            }
            None => {
                let mut panic_msg = format!("{} mutex is deadlocked", guard_name);
                if let Some(path) = $crate::utils::LAST_SUCCESSFUL_LOCK.lock().get(guard_name) {
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
        log::debug!("{} start", name);
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
        .name(format!("Seelen Thread - {}", id))
        .spawn(cb)
        .map_err(|e| format!("Failed to spawn thread: {}", e).into())
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
#[allow(dead_code)]
pub fn convert_file_to_src(path: &str) -> String {
    #[cfg(any(windows, target_os = "android"))]
    let base = "http://asset.localhost/";
    #[cfg(not(any(windows, target_os = "android")))]
    let base = "asset://localhost/";
    let encoded: String = url::form_urlencoded::byte_serialize(path.as_bytes()).collect();
    format!("{base}{encoded}")
}
