pub mod constants;
pub mod discord;
pub mod icon_extractor;
pub mod integrity;
pub mod lock_free;
pub mod pwsh;
pub mod updater;
pub mod virtual_desktop;
mod winver;

use base64::Engine;
use seelen_core::resource::WidgetId;
use uuid::Uuid;
pub use winver::*;

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc, LazyLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use itertools::Itertools;
use parking_lot::Mutex;
use windows::{
    core::GUID,
    Win32::UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT},
};

use crate::{error::Result, get_tokio_handle};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WidgetWebviewLabel {
    /// this should be used as the real webview label
    pub raw: String,
    /// this is the decoded label, useful for debugging and logging
    decoded: String,
    /// widget id from this label was created
    pub widget_id: WidgetId,
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
            widget_id: WidgetId::from(widget_id),
        }
    }

    pub fn try_from_raw(raw: &str) -> Result<Self> {
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(raw)?;
        let decoded = String::from_utf8(decoded)?;
        let widget_id = WidgetId::from(decoded.split('?').next().expect("Invalid label"));

        Ok(Self {
            raw: raw.to_string(),
            decoded,
            widget_id,
        })
    }
}

impl std::fmt::Display for WidgetWebviewLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.decoded)
    }
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

pub struct Debouncer {
    delay: Duration,
    task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl Debouncer {
    /// Create a new debouncer with a delay.
    pub fn new(delay: Duration) -> Self {
        Debouncer {
            delay,
            task: Arc::new(Mutex::new(None)),
        }
    }

    /// Call the function after the delay.
    pub fn call<F, Fut, R>(&self, f: F)
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = R> + Send + 'static,
    {
        let mut task = self.task.lock();

        // Cancel existing timer if it exists
        if let Some(handle) = task.take() {
            handle.abort();
        }

        // Set a new timer
        let delay = self.delay;
        *task = Some(get_tokio_handle().spawn(async move {
            tokio::time::sleep(delay).await;
            f().await;
        }));
    }
}
