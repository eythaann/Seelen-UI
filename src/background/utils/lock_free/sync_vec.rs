use crate::utils::lock_free::TracedMutex;

/// Wrapper for `Mutex<Vec<T>>` with simplifies the API and prevents deadlocks
#[derive(Debug)]
pub struct SyncVec<T>(TracedMutex<Vec<T>>);

#[allow(dead_code)]
impl<T> SyncVec<T> {
    pub fn new() -> Self {
        Self(TracedMutex::new(Vec::new()))
    }

    #[inline]
    #[track_caller]
    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.0.lock().contains(item)
    }

    #[inline]
    #[track_caller]
    pub fn len(&self) -> usize {
        self.0.lock().len()
    }

    #[inline]
    #[track_caller]
    pub fn push(&self, item: T) {
        self.0.lock().push(item);
    }

    #[inline]
    #[track_caller]
    pub fn insert(&self, index: usize, item: T) {
        self.0.lock().insert(index, item);
    }

    #[inline]
    #[track_caller]
    pub fn any(&self, f: impl FnMut(&T) -> bool) -> bool {
        self.0.lock().iter().any(f)
    }

    /// Atomically checks whether an item matching `pred` is already present and, if not,
    /// inserts the item produced by `make`. The check and insert happen under a single lock
    /// acquisition, avoiding the TOCTOU race of calling `any`/`contains` followed by `push`.
    /// Returns `true` if the item was inserted.
    #[inline]
    #[track_caller]
    pub fn get_or_insert_with(&self, pred: impl Fn(&T) -> bool, make: impl FnOnce() -> T) -> bool {
        let mut items = self.0.lock();
        if items.iter().any(pred) {
            return false;
        }
        items.push(make());
        true
    }

    #[inline]
    #[track_caller]
    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&mut T),
    {
        self.0.lock().iter_mut().for_each(f);
    }

    #[inline]
    #[track_caller]
    pub fn map<F, R>(&self, f: F) -> Vec<R>
    where
        F: FnMut(&mut T) -> R,
    {
        self.0.lock().iter_mut().map(f).collect()
    }

    #[inline]
    #[track_caller]
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.0.lock().retain(f);
    }

    #[inline]
    #[track_caller]
    pub fn clear(&self) {
        self.0.lock().clear();
    }

    #[inline]
    #[track_caller]
    pub fn drain(&self) -> Vec<T> {
        self.0.lock().drain(..).collect()
    }

    #[inline]
    #[track_caller]
    pub fn replace(&self, value: Vec<T>) {
        *self.0.lock() = value;
    }

    #[inline]
    #[track_caller]
    pub fn sort_by<F>(&self, compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        self.0.lock().sort_by(compare);
    }
}

impl<T: Clone> SyncVec<T> {
    #[inline]
    #[track_caller]
    pub fn to_vec(&self) -> Vec<T> {
        self.0.lock().clone()
    }

    #[inline]
    #[track_caller]
    pub fn find_and_clone(&self, f: impl Fn(&T) -> bool) -> Option<T> {
        let items = self.0.lock();
        for item in items.iter() {
            if f(item) {
                return Some(item.clone());
            }
        }
        None
    }
}

impl<T> From<Vec<T>> for SyncVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(TracedMutex::new(value))
    }
}

impl<T> Default for SyncVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
