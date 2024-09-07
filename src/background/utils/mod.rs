pub mod ahk;
pub mod constants;
pub mod pwsh;
pub mod virtual_desktop;
mod winver;

pub use winver::*;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

use itertools::Itertools;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::{
    core::GUID,
    Win32::{
        Foundation::{HANDLE, RECT},
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
    if a.right < b.left || a.left > b.right || a.bottom < b.top || a.top > b.bottom {
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
            let rfid = GUID::from(guid);
            let string_path = unsafe {
                SHGetKnownFolderPath(&rfid as _, KF_FLAG_DEFAULT, HANDLE(0))?.to_string()?
            };

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
        if $crate::utils::TRACE_LOCK_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            let guard_name = stringify!($mutex);
            if guard.is_none() {
                let map = $crate::utils::LAST_SUCCESSFUL_LOCK.lock();
                log::info!(
                    "Error: Last successful lock for mutex {} was at: {:?}",
                    guard_name,
                    map.get(guard_name)
                );
            } else {
                let mut map = $crate::utils::LAST_SUCCESSFUL_LOCK.lock();
                let location = format!("{}:{}", file!(), line!());
                log::trace!("{} lock acquired at {}", guard_name, location);
                map.insert(guard_name.to_owned(), location);
            }
        }

        guard.expect("Mutex was poisoned")
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
