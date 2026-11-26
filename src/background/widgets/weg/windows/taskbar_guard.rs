//! Loads the `taskbar_guard` component of `sluhk.dll` into `explorer.exe` for
//! `Shell_TrayWnd` and `Shell_SecondaryTrayWnd`. See `src/hook_dll/src/taskbar_guard.rs` and
//! `slu_ipc::taskbar_guard` for the in-process subclass and protocol.

use std::{
    collections::{HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::{Path, PathBuf},
    sync::{LazyLock, OnceLock, mpsc},
    thread,
    time::Duration,
};

use slu_ipc::{
    messages::SvcAction,
    taskbar_guard::{ATTACH_MESSAGE, DETACH_MESSAGE, HOOK_PROC},
};
use slu_utils::checksums::calculate_sha256;
use windows::{
    Win32::{
        Foundation::{HANDLE, HMODULE, HWND, LPARAM, LRESULT, WPARAM},
        Storage::FileSystem::{FILE_SHARE_READ, GetFinalPathNameByHandleW, VOLUME_NAME_DOS},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::WindowsAndMessaging::{
            FindWindowExW, HHOOK, HOOKPROC, IsWindow, RegisterWindowMessageW, SMTO_NORMAL,
            SendMessageTimeoutW, SetWindowsHookExW, WH_CALLWNDPROC,
        },
    },
    core::{HSTRING, PCSTR, w},
};
use windows_core::Owned;

use crate::{
    cli::ServicePipe,
    error::{Result, ResultLogExt},
    utils::constants::SEELEN_COMMON,
    windows_api::WindowsApi,
};

const DLL_NAME: &str = "sluhk.dll";
const DLL_FOLDER: &str = "taskbar-guard";
const SEND_TIMEOUT_MS: u32 = 1500;
const MAX_ATTACH_RETRIES: u32 = 5;
const ATTACH_RETRY_DELAY: Duration = Duration::from_secs(1);
const MAX_DETACH_ATTEMPTS: u32 = 5;
const DETACH_RETRY_DELAY: Duration = Duration::from_millis(300);

static GUARD_TX: OnceLock<mpsc::Sender<GuardCommand>> = OnceLock::new();

/// The component loaded in Seelen (for hooking) and the hooked taskbar threads.
struct Injection {
    /// Dropped first: unhooking before the module is freed and the DLL file lock released.
    threads: HashMap<u32, HookedThread>,
    hook_proc: HOOKPROC,
    module: Owned<HMODULE>,
    _locked_file: File,
}

impl Injection {
    fn load() -> Result<Self> {
        let (path, locked_file) = prepare_dll()?;
        let module = unsafe { Owned::new(LoadLibraryW(&HSTRING::from(path.as_os_str()))?) };
        let name = std::ffi::CString::new(HOOK_PROC).map_err(|_| "invalid hook procedure name")?;
        let proc = unsafe { GetProcAddress(*module, PCSTR(name.as_ptr() as _)) }
            .ok_or("the taskbar guard hook procedure is missing from the DLL")?;
        // SAFETY: the export has the HOOKPROC signature.
        let hook_proc: HOOKPROC = Some(unsafe {
            std::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT,
            >(proc)
        });
        log::info!("Taskbar guard component loaded from {}", path.display());
        Ok(Self {
            threads: HashMap::new(),
            hook_proc,
            module,
            _locked_file: locked_file,
        })
    }

    /// Hooks the taskbar window's thread if needed. Returns whether the window is newly tracked.
    fn track(&mut self, hwnd: isize) -> Result<bool> {
        let (_pid, thread_id) = WindowsApi::window_thread_process_id(HWND(hwnd as _));
        if thread_id == 0 {
            return Ok(false);
        }
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            return Ok(thread.windows.insert(hwnd));
        }
        let hook = unsafe {
            SetWindowsHookExW(
                WH_CALLWNDPROC,
                self.hook_proc,
                Some((*self.module).into()),
                thread_id,
            )?
        };
        self.threads.insert(
            thread_id,
            HookedThread {
                _hook: unsafe { Owned::new(hook) },
                windows: HashSet::from([hwnd]),
            },
        );
        Ok(true)
    }

    fn forget(&mut self, hwnd: isize) {
        self.threads.retain(|_, thread| {
            thread.windows.remove(&hwnd);
            !thread.windows.is_empty()
        });
    }

    fn prune_dead(&mut self) {
        self.threads.retain(|_, thread| {
            thread
                .windows
                .retain(|hwnd| unsafe { IsWindow(Some(HWND(*hwnd as _))) }.as_bool());
            !thread.windows.is_empty()
        });
    }

    fn windows(&self) -> Vec<isize> {
        self.threads
            .values()
            .flat_map(|thread| thread.windows.iter().copied())
            .collect()
    }
}

struct HookedThread {
    _hook: Owned<HHOOK>,
    windows: HashSet<isize>,
}

enum GuardCommand {
    Attach,
    Detach(mpsc::Sender<bool>),
}

/// Registered messages shared with `sluhk.dll`.
struct Messages {
    attach: u32,
    detach: u32,
}

impl Messages {
    fn get() -> &'static Self {
        static MESSAGES: LazyLock<Messages> = LazyLock::new(|| unsafe {
            Messages {
                attach: RegisterWindowMessageW(&HSTRING::from(ATTACH_MESSAGE)),
                detach: RegisterWindowMessageW(&HSTRING::from(DETACH_MESSAGE)),
            }
        });
        &MESSAGES
    }
}

#[derive(Clone, Copy)]
enum RetryState {
    None,
    Attach(u32),
    Detach,
}

fn guard_tx() -> &'static mpsc::Sender<GuardCommand> {
    GUARD_TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("Taskbar Guard".to_string())
            .spawn(move || run_worker(rx))
            .expect("Failed to spawn Taskbar Guard worker");
        tx
    })
}

/// Attaches the in-process taskbar guard to all native taskbar windows asynchronously.
pub fn attach_taskbar_guard() {
    let _ = guard_tx().send(GuardCommand::Attach);
}

/// Detaches the in-process taskbar guard from all native taskbar windows synchronously.
///
/// Returns an error if any taskbar window fails to acknowledge detach within the timeout; the
/// background worker continues retrying and restores the native taskbar once detached.
pub fn detach_taskbar_guard() -> Result<()> {
    let Some(guard_tx) = GUARD_TX.get() else {
        return Ok(());
    };
    let (tx, rx) = mpsc::channel();
    guard_tx
        .send(GuardCommand::Detach(tx))
        .map_err(|_| "Taskbar guard worker is not running")?;
    match rx.recv() {
        Ok(true) => Ok(()),
        Ok(false) => Err("Timed out waiting for native taskbar guard to detach".into()),
        Err(_) => Err("Taskbar guard worker stopped before confirming detach".into()),
    }
}

fn run_worker(rx: mpsc::Receiver<GuardCommand>) {
    let mut injection: Option<Injection> = None;
    let mut load_logged_err = false;
    let mut retry = RetryState::None;
    loop {
        let cmd = match retry {
            RetryState::None => {
                let Ok(cmd) = rx.recv() else {
                    break;
                };
                cmd
            }
            RetryState::Attach(left) => match rx.recv_timeout(ATTACH_RETRY_DELAY) {
                Ok(cmd) => cmd,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    retry = if attach_all(&mut injection, &mut load_logged_err) || left <= 1 {
                        RetryState::None
                    } else {
                        RetryState::Attach(left - 1)
                    };
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            },
            RetryState::Detach => match rx.recv_timeout(ATTACH_RETRY_DELAY) {
                Ok(cmd) => cmd,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if !detach_pass(&mut injection) {
                        continue;
                    }
                    retry = RetryState::None;
                    match rx.try_recv() {
                        Ok(cmd) => cmd,
                        Err(mpsc::TryRecvError::Empty) => {
                            ServicePipe::request(SvcAction::RestoreNativeTaskbar).log_error();
                            continue;
                        }
                        Err(mpsc::TryRecvError::Disconnected) => break,
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            },
        };

        match cmd {
            GuardCommand::Attach => {
                retry = if attach_all(&mut injection, &mut load_logged_err) {
                    RetryState::None
                } else {
                    RetryState::Attach(MAX_ATTACH_RETRIES)
                };
            }
            GuardCommand::Detach(done) => {
                let detached = detach_all(&mut injection);
                retry = if detached {
                    RetryState::None
                } else {
                    RetryState::Detach
                };
                load_logged_err = false;
                let _ = done.send(detached);
            }
        }
    }
}

fn attach_all(injection: &mut Option<Injection>, load_logged_err: &mut bool) -> bool {
    let handles = taskbar_windows();
    if handles.is_empty() {
        return false;
    }

    if injection.is_none() {
        match Injection::load() {
            Ok(loaded) => *injection = Some(loaded),
            Err(err) => {
                if !*load_logged_err {
                    *load_logged_err = true;
                    log::warn!("Taskbar guard unavailable: {err}");
                }
                return false;
            }
        }
    }

    let Some(inj) = injection.as_mut() else {
        return false;
    };
    inj.prune_dead();

    let pid = std::process::id() as usize;
    let msg = Messages::get().attach;
    let mut all_attached = true;
    for hwnd in handles {
        match inj.track(hwnd) {
            Ok(true) => {
                if !send(hwnd, msg, pid) {
                    log::warn!(
                        "Taskbar window 0x{hwnd:x} did not acknowledge taskbar guard attach"
                    );
                    inj.forget(hwnd);
                    all_attached = false;
                }
            }
            Ok(false) => {}
            Err(err) => {
                log::warn!("Failed to hook taskbar window 0x{hwnd:x}: {err}");
                all_attached = false;
            }
        }
    }
    all_attached
}

fn detach_all(injection: &mut Option<Injection>) -> bool {
    for attempt in 0..MAX_DETACH_ATTEMPTS {
        if detach_pass(injection) {
            return true;
        }
        if attempt + 1 < MAX_DETACH_ATTEMPTS {
            thread::sleep(DETACH_RETRY_DELAY);
        }
    }
    false
}

fn detach_pass(injection: &mut Option<Injection>) -> bool {
    let Some(inj) = injection.as_mut() else {
        return true;
    };
    inj.prune_dead();
    let mut targets: HashSet<isize> = taskbar_windows().into_iter().collect();
    targets.extend(inj.windows());
    let msg = Messages::get().detach;
    let mut failed = false;
    for hwnd in targets {
        if !unsafe { IsWindow(Some(HWND(hwnd as _))) }.as_bool() {
            inj.forget(hwnd);
            continue;
        }
        if send(hwnd, msg, 0) {
            inj.forget(hwnd);
        } else {
            log::warn!("Taskbar window 0x{hwnd:x} did not acknowledge taskbar guard detach");
            let _ = inj.track(hwnd);
            failed = true;
        }
    }
    if !failed {
        // Only unhook and free once every live target has removed its subclass.
        *injection = None;
    }
    !failed
}

fn taskbar_windows() -> Vec<isize> {
    let mut found = Vec::new();
    for class in [w!("Shell_TrayWnd"), w!("Shell_SecondaryTrayWnd")] {
        let mut after: Option<HWND> = None;
        while let Ok(hwnd) = unsafe { FindWindowExW(None, after, class, None) } {
            if hwnd.is_invalid() {
                break;
            }
            found.push(hwnd.0 as isize);
            after = Some(hwnd);
        }
    }
    found
}

fn send(hwnd: isize, message: u32, wparam: usize) -> bool {
    let taken = unsafe {
        SendMessageTimeoutW(
            HWND(hwnd as _),
            message,
            WPARAM(wparam),
            LPARAM(0),
            SMTO_NORMAL,
            SEND_TIMEOUT_MS,
            None,
        )
    };
    taken.0 != 0
}

fn prepare_dll() -> Result<(PathBuf, File)> {
    let exe = std::env::current_exe()?;
    let source = exe
        .parent()
        .ok_or("Failed to get executable directory")?
        .join(DLL_NAME);
    if !source.exists() {
        return Err(format!("DLL not found at {}", source.display()).into());
    }
    let content = std::fs::read(&source)?;
    let hash = calculate_sha256(&content);
    let primary_name = format!("sluhk-{hash}.dll");

    let dir = SEELEN_COMMON.app_temp_dir().join(DLL_FOLDER);
    std::fs::create_dir_all(&dir)?;
    let primary = dir.join(&primary_name);

    if let Some(locked) = open_verified_dll(&primary, &content) {
        let resolved = final_path(&locked)?;
        remove_old_copies(&dir, &primary_name);
        return Ok((resolved, locked));
    }

    let _ = std::fs::remove_file(&primary);
    if write_exclusive(&primary, &content).is_ok()
        && let Some(locked) = open_verified_dll(&primary, &content)
    {
        let resolved = final_path(&locked)?;
        remove_old_copies(&dir, &primary_name);
        return Ok((resolved, locked));
    }

    // If the cached path was planted or locked, fall back to a process-unique name before
    // sharing the file for read-only DLL loading.
    let pid = std::process::id();
    for idx in 0u32..8 {
        let name = format!("sluhk-{hash}-{pid}-{idx}.dll");
        let candidate = dir.join(&name);
        if write_exclusive(&candidate, &content).is_ok()
            && let Some(locked) = open_verified_dll(&candidate, &content)
        {
            let resolved = final_path(&locked)?;
            remove_old_copies(&dir, &name);
            return Ok((resolved, locked));
        }
    }

    Err(format!(
        "Failed to stage verified taskbar guard DLL in {}",
        dir.display()
    )
    .into())
}

fn write_exclusive(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .share_mode(0)
        .open(path)?;
    file.write_all(content)?;
    file.sync_all()
}

/// Opens `path` with `FILE_SHARE_READ` (blocking writes, deletes, and renames while held) and
/// confirms the handle's exact bytes match `expected`.
fn open_verified_dll(path: &Path, expected: &[u8]) -> Option<File> {
    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ.0)
        .open(path)
        .ok()?;
    let mut actual = Vec::with_capacity(expected.len());
    file.read_to_end(&mut actual).ok()?;
    (actual == expected).then_some(file)
}

fn final_path(file: &File) -> Result<PathBuf> {
    let mut buf = vec![0u16; 512];
    loop {
        let len = unsafe {
            GetFinalPathNameByHandleW(HANDLE(file.as_raw_handle()), &mut buf, VOLUME_NAME_DOS)
        } as usize;
        if len == 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if len < buf.len() {
            let text = String::from_utf16_lossy(&buf[..len]);
            let normalized = text
                .strip_prefix(r"\\?\")
                .filter(|rest| rest.as_bytes().get(1) == Some(&b':'))
                .unwrap_or(&text);
            return Ok(PathBuf::from(normalized));
        }
        buf.resize(len + 1, 0);
    }
}

fn remove_old_copies(dir: &Path, keep: &str) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name != keep && name.starts_with("sluhk-") && name.ends_with(".dll") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "seelen-taskbar-guard-{label}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn verified_dll_is_locked_against_writes_while_held() {
        let dir = TempDir::new("lock");
        let path = dir.0.join("sluhk-test.dll");
        let bytes = b"valid-sluhk-payload";
        write_exclusive(&path, bytes).expect("write exclusive");

        let locked = open_verified_dll(&path, bytes).expect("matching cached DLL opens");
        assert!(
            OpenOptions::new().write(true).open(&path).is_err(),
            "writes must be blocked while the verified read-shared handle is held"
        );
        drop(locked);
        assert!(OpenOptions::new().write(true).open(&path).is_ok());
    }

    #[test]
    fn tampered_cached_dll_is_rejected() {
        let dir = TempDir::new("tampered");
        let path = dir.0.join("sluhk-test.dll");
        write_exclusive(&path, b"planted-bytes").expect("write planted");

        assert!(open_verified_dll(&path, b"expected-bytes").is_none());
    }
}
