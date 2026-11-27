#[cfg(windows)]
mod windows;

use std::sync::{
    LazyLock,
    atomic::{AtomicBool, Ordering},
};

use crate::error::Result;

use super::domain::TaskbarSystem;

#[cfg(windows)]
type Platform = windows::WindowsTaskbar;

/// Delay before each check of a hide pass. The first checks are close together because the OS
/// shell usually re-shows the taskbar right after it is hidden; the last ones are spaced out to
/// catch it re-showing later (after its own startup, display changes, etc.). Checks are cheap:
/// they only act on taskbars that are shown.
const HIDE_PASS_DELAYS_MS: [u64; 14] =
    [0, 50, 50, 50, 50, 50, 50, 50, 50, 50, 250, 500, 1000, 2000];

pub struct TaskbarManager {
    platform: Platform,
    /// Tracks whether we actually hid the native taskbar, so we only restore it
    /// when it was hidden by us (avoids unnecessary restores on settings changes/shutdown).
    hidden: AtomicBool,
    /// Set by every hide request and consumed by the worker before each pass, so a request
    /// that arrives while a pass is running is never lost: it schedules one more pass.
    hide_requested: AtomicBool,
    /// Only one worker thread runs hide passes at a time.
    hide_worker_running: AtomicBool,
}

impl TaskbarManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<TaskbarManager> = LazyLock::new(TaskbarManager::create);
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            hidden: AtomicBool::new(false),
            hide_requested: AtomicBool::new(false),
            hide_worker_running: AtomicBool::new(false),
        }
    }

    pub fn hide(&'static self) {
        self.hidden.store(true, Ordering::SeqCst);
        self.hide_requested.store(true, Ordering::SeqCst);
        if self.hide_worker_running.swap(true, Ordering::SeqCst) {
            // the running worker will see `hide_requested` and run another pass
            return;
        }
        std::thread::spawn(move || {
            loop {
                while self.hide_requested.swap(false, Ordering::SeqCst) {
                    self.run_hide_pass();
                }
                self.hide_worker_running.store(false, Ordering::SeqCst);
                // A request may have arrived after the last swap but before releasing the worker
                // flag; its caller saw the worker as running and returned, so pick it up here
                // unless another worker already did.
                if !self.hide_requested.load(Ordering::SeqCst)
                    || self.hide_worker_running.swap(true, Ordering::SeqCst)
                {
                    break;
                }
            }
        });
    }

    pub fn restore(&self) -> Result<()> {
        if self
            .hidden
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }
        self.platform.restore()
    }

    fn is_hidden(&self) -> bool {
        self.hidden.load(Ordering::SeqCst)
    }

    fn run_hide_pass(&self) {
        let mut focus_moved = false;
        for delay in HIDE_PASS_DELAYS_MS {
            if delay > 0 {
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
            if !self.is_hidden() {
                // restored meanwhile; a later hide request schedules a new pass
                return;
            }
            match self.platform.hide(!focus_moved, || self.is_hidden()) {
                Ok(moved) => focus_moved |= moved,
                Err(err) => log::error!("Failed to hide the native taskbar: {err:?}"),
            }
        }
    }
}
