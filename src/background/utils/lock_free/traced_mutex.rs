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
    #[cfg(test)]
    timeout: std::time::Duration,
}

const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

impl<T> TracedMutex<T> {
    /// Creates a new TracedMutex with the given value
    pub fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
            last_lock_location: AtomicPtr::new(std::ptr::null_mut()),
            #[cfg(test)]
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Test-only constructor allowing a shorter timeout so deadlock tests don't
    /// have to block for the real (5s) production timeout.
    #[cfg(test)]
    fn with_timeout(value: T, timeout: std::time::Duration) -> Self {
        Self {
            inner: Mutex::new(value),
            last_lock_location: AtomicPtr::new(std::ptr::null_mut()),
            timeout,
        }
    }

    #[inline]
    #[cfg(test)]
    fn timeout(&self) -> std::time::Duration {
        self.timeout
    }

    #[inline]
    #[cfg(not(test))]
    fn timeout(&self) -> std::time::Duration {
        DEFAULT_TIMEOUT
    }

    /// Locks the mutex and records the caller's location.
    ///
    /// The `#[track_caller]` attribute ensures that `Location::caller()` returns
    /// the location where `lock()` was called, not the location inside this function.
    /// This propagates through to `panic_timeout` (also `#[track_caller]`), so a
    /// timeout panic reports the original call site, not a line inside this module.
    ///
    /// # Panics
    ///
    /// This function will panic if the mutex is already locked by the current thread (deadlock).
    /// The panic will include:
    /// - The current caller location (where this call to `lock()` is blocked)
    /// - The last recorded lock location (if any)
    #[inline]
    #[track_caller]
    pub fn lock<'a>(&'a self) -> MutexGuard<'a, T> {
        // Try to acquire the lock with a timeout to detect potential deadlocks
        match self.inner.try_lock_for(self.timeout()) {
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
    #[track_caller]
    fn panic_timeout(&self) -> ! {
        let last = unsafe { self.last_lock_location.load(Ordering::Acquire).as_ref() };

        match last {
            Some(last) => panic!(
                "{:?}",
                AppError::from(format!(
                    "Mutex lock timed out after {:?}.\n  Last lock acquired at: {last}",
                    self.timeout(),
                ))
            ),
            None => panic!(
                "{:?}",
                AppError::from(format!("Mutex lock timed out after {:?}.", self.timeout()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    /// `panics_with_blocked_and_last_lock_locations_on_timeout` installs a process-global
    /// panic hook while it expects a panic on this thread. Since tests run in parallel by
    /// default, any other test panicking concurrently (e.g. the intentional panic in
    /// `panics_without_last_location_when_never_locked_successfully`) would be captured by
    /// that hook instead, corrupting the recorded location. Serialize the two tests that
    /// deliberately panic to avoid this race.
    static PANIC_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn locks_and_unlocks() {
        let mutex = TracedMutex::new(0);
        *mutex.lock() = 42;
        assert_eq!(*mutex.lock(), 42);
    }

    #[test]
    fn records_last_lock_location() {
        let mutex = TracedMutex::new(0);
        let line_of_lock = line!() + 1;
        let _guard = mutex.lock();
        let last = unsafe { mutex.last_lock_location.load(Ordering::Acquire).as_ref() }
            .expect("a successful lock must record its location");
        assert!(last.file().ends_with("traced_mutex.rs"));
        assert_eq!(last.line(), line_of_lock);
    }

    #[test]
    fn panics_with_blocked_and_last_lock_locations_on_timeout() {
        let _serialize = PANIC_TEST_LOCK.lock().unwrap();
        let mutex = Arc::new(TracedMutex::with_timeout(0, Duration::from_millis(50)));

        // Take the lock on another thread and hold it, recording "last lock" here.
        let held = Arc::clone(&mutex);
        let (tx, rx) = std::sync::mpsc::channel();
        let holder = std::thread::spawn(move || {
            let _guard = held.lock();
            tx.send(()).unwrap();
            std::thread::sleep(Duration::from_secs(1));
        });
        rx.recv().unwrap();

        // This call should block until the timeout and then panic. The panic's own
        // location (as seen by the panic hook, not the message text) must point at
        // *this* call site, proving `#[track_caller]` propagated through `panic_timeout`
        // instead of pointing inside traced_mutex.rs.
        let blocked_line = line!() + 11;
        let panicked_at: Arc<std::sync::Mutex<Option<(String, u32)>>> = Arc::default();
        let panicked_at_clone = Arc::clone(&panicked_at);
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if let Some(location) = info.location() {
                *panicked_at_clone.lock().unwrap() =
                    Some((location.file().to_string(), location.line()));
            }
        }));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock();
        }));
        std::panic::set_hook(previous_hook);

        assert!(result.is_err(), "lock() must panic after timing out");
        let message = result
            .err()
            .and_then(|payload| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        assert!(message.contains("timed out"));
        assert!(message.contains("Last lock acquired at"));

        let (file, line) = panicked_at
            .lock()
            .unwrap()
            .clone()
            .expect("panic hook must receive a location");
        assert!(file.ends_with("traced_mutex.rs"));
        assert_eq!(
            line, blocked_line,
            "expected the panic location to point at line {blocked_line} (this test's `mutex.lock()` call)"
        );

        holder.join().unwrap();
    }

    #[test]
    fn panics_without_last_location_when_never_locked_successfully() {
        let _serialize = PANIC_TEST_LOCK.lock().unwrap();
        let mutex: TracedMutex<i32> = TracedMutex::with_timeout(0, Duration::from_millis(50));
        // Poison-free: hold the underlying lock directly without going through
        // `TracedMutex::lock`, so `last_lock_location` is never populated.
        let _raw_guard = mutex.inner.lock();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock();
        }));

        assert!(result.is_err());
        let message = result
            .err()
            .and_then(|payload| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        assert!(message.contains("timed out"));
        assert!(!message.contains("Last lock acquired at"));
    }
}
