use crate::utils::lock_free::TracedMutex;

/// Wrapper for `Mutex<Vec<T>>` with simplifies the API and prevents deadlocks
#[derive(Debug)]
pub struct SyncVec<T>(TracedMutex<Vec<T>>);

#[allow(dead_code)]
impl<T> SyncVec<T> {
    pub fn new() -> Self {
        Self(TracedMutex::new(Vec::new()))
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        self.0.lock().contains(item)
    }

    pub fn len(&self) -> usize {
        self.0.lock().len()
    }

    pub fn push(&self, item: T) {
        self.0.lock().push(item);
    }

    pub fn any(&self, f: impl FnMut(&T) -> bool) -> bool {
        self.0.lock().iter().any(f)
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&mut T),
    {
        self.0.lock().iter_mut().for_each(f);
    }

    pub fn map<F, R>(&self, f: F) -> Vec<R>
    where
        F: FnMut(&mut T) -> R,
    {
        self.0.lock().iter_mut().map(f).collect()
    }

    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.0.lock().retain(f);
    }

    pub fn clear(&self) {
        self.0.lock().clear();
    }

    pub fn drain(&self) -> Vec<T> {
        self.0.lock().drain(..).collect()
    }

    pub fn replace(&self, value: Vec<T>) {
        *self.0.lock() = value;
    }

    pub fn sort_by<F>(&self, compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        self.0.lock().sort_by(compare);
    }
}

impl<T: Clone> SyncVec<T> {
    pub fn to_vec(&self) -> Vec<T> {
        self.0.lock().clone()
    }

    pub fn find(&self, f: impl Fn(&T) -> bool) -> Option<T> {
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
