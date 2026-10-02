use std::time::Instant;

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Threading::{
        CREATE_WAITABLE_TIMER_HIGH_RESOLUTION, CreateWaitableTimerExW, GetCurrentThread, INFINITE,
        SetThreadPriority, SetWaitableTimer, THREAD_PRIORITY_HIGHEST, WaitForSingleObject,
    },
};

use crate::error::Result;

/// High-resolution waitable timer (~100ns precision vs ~15.6ms for thread::sleep).
pub struct FrameTimer {
    handle: HANDLE,
}

impl FrameTimer {
    pub fn new() -> Result<Self> {
        let handle = unsafe {
            CreateWaitableTimerExW(
                None,
                None,
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                0x001F0003,
            )?
        };
        Ok(Self { handle })
    }

    /// Blocks until `target`. Returns immediately if it is already in the past.
    pub fn sleep_until(&self, target: Instant) {
        let Some(remaining) = target.checked_duration_since(Instant::now()) else {
            return;
        };
        // Negative due time = relative, in 100ns units.
        let due_100ns = -(remaining.as_nanos() as i64 / 100).max(1);
        unsafe {
            let _ = SetWaitableTimer(self.handle, &due_100ns, 0, None, None, false);
            WaitForSingleObject(self.handle, INFINITE);
        }
    }
}

impl Drop for FrameTimer {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

pub fn boost_current_thread_priority() {
    unsafe {
        let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
    }
}
