use parking_lot::{Mutex, MutexGuard};
use rust_i18n::AtomicStr;
use std::panic::Location;

use crate::error::AppError;

/// A wrapper around parking_lot::Mutex that tracks the last location where it was locked.
/// This is useful for debugging deadlocks and understanding lock contention.
pub struct TracedMutex<T> {
    inner: Mutex<T>,
    last_lock_location: AtomicStr,
}

impl<T> TracedMutex<T> {
    /// Creates a new TracedMutex with the given value
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
            last_lock_location: AtomicStr::new(""),
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
        // let current_location = Location::caller();

        // Try to acquire the lock with a timeout to detect potential deadlocks
        match self.inner.try_lock_for(std::time::Duration::from_secs(5)) {
            Some(guard) => {
                let location = Location::caller();
                self.last_lock_location.replace(format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                ));
                guard
            }
            None => {
                // Lock is already held, gather information and panic
                let msg = format!(
                    "Mutex is deadlocked, Last lock location: {}",
                    self.last_lock_location
                );
                panic!("{:?}", AppError::from(msg));
            }
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
