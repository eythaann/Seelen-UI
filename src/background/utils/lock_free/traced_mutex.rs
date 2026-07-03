use parking_lot::{Mutex, MutexGuard};
use std::{
    panic::Location,
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::error::AppError;

/// A wrapper around parking_lot::Mutex that tracks the last location where it was locked.
/// This is useful for debugging deadlocks and understanding lock contention.
pub struct TracedMutex<T> {
    inner: Mutex<T>,
    last_lock_location: AtomicPtr<Location<'static>>,
}

impl<T> TracedMutex<T> {
    /// Creates a new TracedMutex with the given value
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
            last_lock_location: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    /// Locks the mutex and records the caller's location.
    ///
    /// The `#[track_caller]` attribute ensures that `Location::caller()` returns
    /// the location where `trace_lock()` was called, not the location inside this function.
    ///
    /// # Panics
    ///
    /// This function will panic if the mutex is already locked by the current thread (deadlock).
    /// The panic will include:
    /// - The current caller location (where trace_lock was called)
    /// - The last recorded lock location (if any)
    #[track_caller]
    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        // Try to acquire the lock with a timeout to detect potential deadlocks
        match self.inner.try_lock_for(std::time::Duration::from_secs(5)) {
            Some(guard) => {
                let location = Location::caller();
                self.last_lock_location
                    .store(location as *const _ as *mut _, Ordering::Release);
                guard
            }
            None => self.panic_timeout(),
        }
    }

    #[cold]
    fn panic_timeout(&self) -> ! {
        let last = unsafe { self.last_lock_location.load(Ordering::Acquire).as_ref() };

        match last {
            Some(last) => panic!(
                "{:?}",
                AppError::from(format!(
                    "Mutex lock timed out after 5 seconds.\nLast lock acquired at {}:{}:{}",
                    last.file(),
                    last.line(),
                    last.column(),
                ))
            ),
            None => panic!(
                "{:?}",
                AppError::from("Mutex lock timed out after 5 seconds.")
            ),
        }
    }
}

impl<T: Default> Default for TracedMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

// Implement common traits to make TracedMutex a drop-in replacement
impl<T: std::fmt::Debug> std::fmt::Debug for TracedMutex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.inner.try_lock() {
            Some(guard) => f
                .debug_struct("TracedMutex")
                .field("data", &*guard)
                .finish(),
            None => f
                .debug_struct("TracedMutex")
                .field("data", &"<locked>")
                .finish(),
        }
    }
}

// Safety: TracedMutex can be sent between threads if T can be sent
unsafe impl<T: Send> Send for TracedMutex<T> {}
// Safety: TracedMutex can be shared between threads if T can be sent
unsafe impl<T: Send> Sync for TracedMutex<T> {}
