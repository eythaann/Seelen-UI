#[cfg(windows)]
mod windows;

use std::{
    sync::{
        Condvar, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
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
    /// Signals the hide worker to wake immediately on a new hide or restore request.
    hide_signal: (Mutex<HideWorkerState>, Condvar),
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
            hide_signal: (Mutex::new(HideWorkerState::default()), Condvar::new()),
        }
    }

    pub fn hide(&'static self) {
        self.hidden.store(true, Ordering::SeqCst);
        let (lock, cvar) = &self.hide_signal;
        let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        state.requested = true;
        cvar.notify_one();
        if state.running {
            return;
        }
        state.running = true;
        drop(state);
        std::thread::spawn(move || self.run_hide_worker());
    }

    pub fn restore(&self) -> Result<()> {
        if self
            .hidden
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }
        {
            let _guard = self
                .hide_signal
                .0
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            self.hide_signal.1.notify_one();
        }
        self.platform.restore()
    }

    fn is_hidden(&self) -> bool {
        self.hidden.load(Ordering::SeqCst)
    }

    fn run_hide_worker(&self) {
        let (lock, _) = &self.hide_signal;
        loop {
            {
                let mut state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                if !state.requested || !self.is_hidden() {
                    state.requested = false;
                    state.running = false;
                    return;
                }
                state.requested = false;
            }
            self.run_hide_pass();
        }
    }

    fn run_hide_pass(&self) {
        let (lock, cvar) = &self.hide_signal;
        let mut focus_moved = false;
        let mut idx = 0;
        while idx < HIDE_PASS_DELAYS_MS.len() {
            let delay = HIDE_PASS_DELAYS_MS[idx];
            if delay > 0 {
                let state = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                let (mut state, _) = cvar
                    .wait_timeout_while(state, Duration::from_millis(delay), |s| {
                        !s.requested && self.is_hidden()
                    })
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if !self.is_hidden() {
                    // restored meanwhile; a later hide request schedules a new pass
                    return;
                }
                if state.requested {
                    state.requested = false;
                    drop(state);
                    idx = 0;
                    focus_moved = false;
                    continue;
                }
            }
            if !self.is_hidden() {
                return;
            }
            match self.platform.hide(!focus_moved, || self.is_hidden()) {
                Ok(moved) => focus_moved |= moved,
                Err(err) => log::error!("Failed to hide the native taskbar: {err:?}"),
            }
            idx += 1;
        }
    }
}

#[derive(Debug, Default)]
struct HideWorkerState {
    requested: bool,
    running: bool,
}
