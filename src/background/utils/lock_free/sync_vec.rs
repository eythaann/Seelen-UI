use crate::utils::lock_free::TracedMutex;

/// Wrapper for `Mutex<Vec<T>>` with simplifies the API and prevents deadlocks
pub struct SyncVec<T>(TracedMutex<Vec<T>>);

#[allow(dead_code)]
impl<T> SyncVec<T> {
    pub fn new() -> Self {
        Self(TracedMutex::new(Vec::new()))
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
}

impl<T: Clone> SyncVec<T> {
    pub fn to_vec(&self) -> Vec<T> {
        self.0.lock().clone()
    }
}

impl<T> From<Vec<T>> for SyncVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(TracedMutex::new(value))
    }
}
