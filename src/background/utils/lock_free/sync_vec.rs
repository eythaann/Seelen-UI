use parking_lot::Mutex;

use crate::trace_lock;

/// Wrapper for `Mutex<Vec<T>>` with simplifies the API and prevents deadlocks
pub struct SyncVec<T>(Mutex<Vec<T>>);

#[allow(dead_code)]
impl<T> SyncVec<T> {
    pub fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub fn len(&self) -> usize {
        trace_lock!(self.0).len()
    }

    pub fn push(&self, item: T) {
        trace_lock!(self.0).push(item);
    }

    pub fn any(&self, f: impl FnMut(&T) -> bool) -> bool {
        trace_lock!(self.0).iter().any(f)
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut(&mut T),
    {
        trace_lock!(self.0).iter_mut().for_each(f);
    }

    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        trace_lock!(self.0).retain(f);
    }

    pub fn clear(&self) {
        trace_lock!(self.0).clear();
    }
}

impl<T: Clone> SyncVec<T> {
    pub fn to_vec(&self) -> Vec<T> {
        trace_lock!(self.0).clone()
    }
}

impl<T> From<Vec<T>> for SyncVec<T> {
    fn from(value: Vec<T>) -> Self {
        Self(Mutex::new(value))
    }
}
