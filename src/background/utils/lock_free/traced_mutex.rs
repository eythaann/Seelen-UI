use parking_lot::{Mutex, MutexGuard};
use std::{
    cell::Cell,
    collections::HashMap,
    ops::{Deref, DerefMut},
    panic::Location,
    sync::{
        LazyLock,
        atomic::{AtomicPtr, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use crate::error::AppError;

/// How often a blocked `lock()` re-checks the wait-for graph for a cycle. A thread only
/// joins the graph after its first wait of this length fails, so short contended waits
/// (the vast majority) never touch `WAITERS`. A real deadlock between `TracedMutex`es is
/// reported after roughly one or two of these intervals.
const CYCLE_CHECK_INTERVAL: Duration = Duration::from_millis(250);

/// After this long without a cycle, the holder is assumed to be slow (not deadlocked)
/// and a warning is logged pointing at it.
const SLOW_HOLDER_WARNING: Duration = Duration::from_secs(5);

/// Last resort for hangs the wait-for graph can't see (the holder is blocked on
/// something that is not a `TracedMutex`: std locks, channels, COM, SendMessage...).
/// Generous because, without a cycle, the holder may just be slow.
#[cfg(debug_assertions)]
const HARD_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(not(debug_assertions))]
const HARD_TIMEOUT: Duration = Duration::from_secs(30);

/// Threads currently blocked in `TracedMutex::lock`, keyed by thread tag.
/// Together with each mutex's `owner` this forms the wait-for graph.
static WAITERS: LazyLock<Mutex<HashMap<u64, Waiter>>> = LazyLock::new(Default::default);

static NEXT_THREAD_TAG: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// Cheap unique id per thread (`ThreadId::as_u64` is unstable), assigned on first use.
    /// 0 means "no owner". Const-initialized and without a destructor so it stays accessible
    /// even while thread-locals are being destroyed (a `Drop` there may still lock).
    static THREAD_TAG: Cell<u64> = const { Cell::new(0) };
}

/// A wrapper around parking_lot::Mutex that tracks who holds it and where it was locked.
///
/// A blocked `lock()` distinguishes a deadlock from a slow holder:
/// - If the threads waiting on `TracedMutex`es form a cycle that includes the caller
///   (including locking twice on the same thread), it panics right away with the chain.
/// - Otherwise the holder is just slow: it warns, keeps waiting, and only panics after
///   `HARD_TIMEOUT`, preferring a crash over a frozen app.
pub struct TracedMutex<T> {
    inner: Mutex<T>,
    /// Tag of the thread holding the lock, 0 when unlocked. Only nonzero while that
    /// thread really holds it (stored after acquiring, cleared before releasing).
    owner: AtomicU64,
    last_lock_location: AtomicPtr<Location<'static>>,
    #[cfg(test)]
    timeout: Duration,
}

impl<T> TracedMutex<T> {
    /// Creates a new TracedMutex with the given value
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
            owner: AtomicU64::new(0),
            last_lock_location: AtomicPtr::new(std::ptr::null_mut()),
            #[cfg(test)]
            timeout: HARD_TIMEOUT,
        }
    }

    /// Test-only constructor allowing a shorter timeout so tests don't
    /// have to block for the real hard timeout.
    #[cfg(test)]
    fn with_timeout(value: T, timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::new(value)
        }
    }

    #[inline]
    #[cfg(test)]
    fn timeout(&self) -> Duration {
        self.timeout
    }

    #[inline]
    #[cfg(not(test))]
    fn timeout(&self) -> Duration {
        HARD_TIMEOUT
    }

    /// Locks the mutex and records the caller's location.
    ///
    /// The `#[track_caller]` attribute ensures that `Location::caller()` returns
    /// the location where `lock()` was called, not the location inside this function.
    /// This propagates through to the panic helpers (also `#[track_caller]`), so a
    /// panic reports the original call site, not a line inside this module.
    ///
    /// # Panics
    ///
    /// - Within ~2 * `CYCLE_CHECK_INTERVAL` if waiting would deadlock: the current
    ///   thread already holds this mutex, or is part of a cycle of threads waiting on
    ///   each other's `TracedMutex`es. The message includes the whole chain.
    /// - After `HARD_TIMEOUT` if the lock is still not acquired, even without a cycle.
    #[inline]
    #[track_caller]
    pub fn lock<'a>(&'a self) -> TracedMutexGuard<'a, T> {
        match self.inner.try_lock() {
            Some(guard) => self.on_acquired(guard),
            None => self.lock_slow(),
        }
    }

    #[cold]
    #[track_caller]
    fn lock_slow<'a>(&'a self) -> TracedMutexGuard<'a, T> {
        let me = thread_tag();
        // Joined lazily, after the first full interval: an unregistered thread just ends
        // any chain walked through it, so this can delay detection but never fake a cycle.
        let mut registration: Option<WaitRegistration> = None;
        let start = Instant::now();
        let mut warned = false;

        loop {
            let remaining = self.timeout().saturating_sub(start.elapsed());
            if remaining.is_zero() {
                drop(registration);
                self.panic_timeout(start.elapsed());
            }

            if let Some(guard) = self.inner.try_lock_for(remaining.min(CYCLE_CHECK_INTERVAL)) {
                // Leave the wait-for graph before publishing ownership, so a concurrent
                // cycle check never sees us both waiting on and owning this mutex.
                drop(registration);
                return self.on_acquired(guard);
            }

            if registration.is_none() {
                registration = Some(WaitRegistration::new(me, self, Location::caller()));
            }

            if let Some(chain) = find_cycle(me) {
                drop(registration);
                panic_deadlock(chain);
            }

            if !warned && start.elapsed() >= SLOW_HOLDER_WARNING {
                warned = true;
                log::warn!(
                    "Waiting {:?} for TracedMutex<{}> at {}, no deadlock detected (slow holder?). Holder: {}",
                    start.elapsed(),
                    std::any::type_name::<T>(),
                    Location::caller(),
                    self.describe_holder(),
                );
            }
        }
    }

    #[inline]
    #[track_caller]
    fn on_acquired<'a>(&'a self, guard: MutexGuard<'a, T>) -> TracedMutexGuard<'a, T> {
        self.owner.store(thread_tag(), Ordering::Release);
        self.last_lock_location
            .store(Location::caller() as *const _ as *mut _, Ordering::Release);
        TracedMutexGuard {
            owner: &self.owner,
            guard,
        }
    }

    fn describe_holder(&self) -> String {
        describe_holder(&self.owner, &self.last_lock_location)
    }

    #[cold]
    #[track_caller]
    fn panic_timeout(&self, elapsed: Duration) -> ! {
        panic!(
            "{:?}",
            AppError::from(format!(
                "Mutex lock timed out after {elapsed:?} without a detected deadlock cycle.\n  Holder: {}",
                self.describe_holder(),
            ))
        )
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

/// Guard returned by `TracedMutex::lock`. Clears the owner before releasing the lock.
pub struct TracedMutexGuard<'a, T> {
    owner: &'a AtomicU64,
    guard: MutexGuard<'a, T>,
}

impl<T> Deref for TracedMutexGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T> DerefMut for TracedMutexGuard<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}

impl<T> Drop for TracedMutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        // Runs before `guard` is dropped (fields drop after `Drop::drop`), so the
        // owner is never reported for a mutex that is already free.
        self.owner.store(0, Ordering::Release);
    }
}

/// A thread blocked in `TracedMutex::lock`: a node of the wait-for graph.
struct Waiter {
    /// Cheap to clone (an `Arc`); the name is only read to build a panic message.
    thread: std::thread::Thread,
    waiting_at: &'static Location<'static>,
    type_name: &'static str,
    /// Points into the awaited `TracedMutex`, which outlives this entry: the entry is
    /// removed (under the `WAITERS` lock) before `lock_slow` returns or unwinds.
    owner: *const AtomicU64,
    last_lock_location: *const AtomicPtr<Location<'static>>,
}

// Safety: the raw pointers are only dereferenced while holding the `WAITERS` lock,
// during which the pointed `TracedMutex` is guaranteed alive (see `Waiter::owner`).
unsafe impl Send for Waiter {}

/// Keeps the current thread registered in `WAITERS` while it waits.
struct WaitRegistration {
    me: u64,
}

impl WaitRegistration {
    fn new<T>(me: u64, mutex: &TracedMutex<T>, waiting_at: &'static Location<'static>) -> Self {
        WAITERS.lock().insert(
            me,
            Waiter {
                thread: std::thread::current(),
                waiting_at,
                type_name: std::any::type_name::<T>(),
                owner: &mutex.owner,
                last_lock_location: &mutex.last_lock_location,
            },
        );
        Self { me }
    }
}

impl Drop for WaitRegistration {
    fn drop(&mut self) {
        WAITERS.lock().remove(&self.me);
    }
}

fn thread_tag() -> u64 {
    THREAD_TAG.with(|tag| {
        if tag.get() == 0 {
            tag.set(NEXT_THREAD_TAG.fetch_add(1, Ordering::Relaxed));
        }
        tag.get()
    })
}

fn describe_holder(owner: &AtomicU64, last_lock_location: &AtomicPtr<Location<'static>>) -> String {
    let owner = owner.load(Ordering::Acquire);
    let last = unsafe { last_lock_location.load(Ordering::Acquire).as_ref() };
    match (owner, last) {
        (0, _) => "<none, just released>".to_string(),
        (owner, Some(last)) => format!("thread #{owner}, locked at {last}"),
        (owner, None) => format!("thread #{owner}"),
    }
}

/// Walks the wait-for graph from `me`: the mutex I wait on -> its owner -> the mutex
/// that owner waits on -> ... Returns the chain if it leads back to `me`.
///
/// Holding the `WAITERS` lock freezes the graph's edges: a waiter can't leave it, and
/// a waiter that already acquired its mutex can't publish ownership until it leaves,
/// so every edge seen here is a real "blocked on" / "held by" relation.
fn find_cycle(me: u64) -> Option<Vec<String>> {
    let waiters = WAITERS.lock();
    let mut chain = Vec::new();
    let mut current = me;

    for _ in 0..waiters.len() {
        let waiter = waiters.get(&current)?;
        // Safety: see `Waiter::owner`, we hold the `WAITERS` lock.
        let (owner, last_lock_location) = unsafe { (&*waiter.owner, &*waiter.last_lock_location) };
        let owner_tag = owner.load(Ordering::Acquire);
        if owner_tag == 0 {
            return None;
        }

        chain.push(format!(
            "thread #{current} ({}) waits at {} for TracedMutex<{}>, held by {}",
            waiter.thread.name().unwrap_or("<unnamed>"),
            waiter.waiting_at,
            waiter.type_name,
            describe_holder(owner, last_lock_location),
        ));

        if owner_tag == me {
            return Some(chain);
        }
        current = owner_tag;
    }

    // Either the chain ended on a running thread, or it loops among other threads
    // without including us (each of those will detect and report it themselves).
    None
}

#[cold]
#[track_caller]
fn panic_deadlock(chain: Vec<String>) -> ! {
    panic!(
        "{:?}",
        AppError::from(format!("Deadlock detected:\n  {}", chain.join("\n  ")))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// `panics_with_blocked_and_last_lock_locations_on_timeout` installs a process-global
    /// panic hook while it expects a panic on this thread. Since tests run in parallel by
    /// default, any other test panicking concurrently would be captured by that hook
    /// instead, corrupting the recorded location. Serialize the tests that deliberately
    /// panic to avoid this race.
    static PANIC_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn panic_message(result: std::thread::Result<()>) -> String {
        result
            .err()
            .and_then(|payload| payload.downcast_ref::<String>().cloned())
            .unwrap_or_default()
    }

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
    fn clears_owner_on_unlock() {
        let mutex = TracedMutex::new(0);
        {
            let _guard = mutex.lock();
            assert_eq!(mutex.owner.load(Ordering::Acquire), thread_tag());
        }
        assert_eq!(mutex.owner.load(Ordering::Acquire), 0);
    }

    #[test]
    fn waits_for_slow_holder_without_panicking() {
        let mutex = Arc::new(TracedMutex::with_timeout(0, Duration::from_secs(5)));

        let held = Arc::clone(&mutex);
        let (tx, rx) = std::sync::mpsc::channel();
        let holder = std::thread::spawn(move || {
            let mut guard = held.lock();
            tx.send(()).unwrap();
            // Longer than several cycle checks, but not a deadlock.
            std::thread::sleep(Duration::from_millis(800));
            *guard = 1;
        });
        rx.recv().unwrap();

        assert_eq!(*mutex.lock(), 1);
        holder.join().unwrap();
    }

    #[test]
    fn panics_immediately_on_same_thread_relock() {
        let _serialize = PANIC_TEST_LOCK.lock().unwrap();
        let mutex = TracedMutex::with_timeout(0, Duration::from_secs(30));
        let _guard = mutex.lock();

        let start = Instant::now();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock();
        }));

        assert!(
            start.elapsed() < Duration::from_secs(5),
            "must not wait for the hard timeout"
        );
        let message = panic_message(result);
        assert!(message.contains("Deadlock detected"), "{message}");
    }

    #[test]
    fn panics_immediately_on_cross_thread_cycle() {
        let _serialize = PANIC_TEST_LOCK.lock().unwrap();
        let a = Arc::new(TracedMutex::with_timeout(0, Duration::from_secs(30)));
        let b = Arc::new(TracedMutex::with_timeout(0, Duration::from_secs(30)));
        let barrier = Arc::new(std::sync::Barrier::new(2));

        let spawn = |first: Arc<TracedMutex<i32>>, second: Arc<TracedMutex<i32>>| {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                let start = Instant::now();
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _first = first.lock();
                    barrier.wait();
                    let _second = second.lock();
                }));
                (start.elapsed(), panic_message(result))
            })
        };

        let t1 = spawn(Arc::clone(&a), Arc::clone(&b));
        let t2 = spawn(b, a);
        let results = [t1.join().unwrap(), t2.join().unwrap()];

        // At least one side detects the cycle and panics, releasing its first lock,
        // which lets the other side finish.
        assert!(
            results
                .iter()
                .any(|(_, msg)| msg.contains("Deadlock detected"))
        );
        for (elapsed, _) in &results {
            assert!(
                *elapsed < Duration::from_secs(5),
                "must not wait for the hard timeout"
            );
        }
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
        // *this* call site, proving `#[track_caller]` propagated through the slow path
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

        let message = panic_message(result);
        assert!(message.contains("timed out"), "{message}");
        assert!(message.contains("locked at"), "{message}");

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
    fn panics_without_holder_location_when_never_locked_successfully() {
        let _serialize = PANIC_TEST_LOCK.lock().unwrap();
        let mutex: TracedMutex<i32> = TracedMutex::with_timeout(0, Duration::from_millis(50));
        // Hold the underlying lock directly without going through `TracedMutex::lock`,
        // so neither `owner` nor `last_lock_location` are populated.
        let _raw_guard = mutex.inner.lock();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock();
        }));

        let message = panic_message(result);
        assert!(message.contains("timed out"), "{message}");
        assert!(!message.contains("locked at"), "{message}");
    }
}
