use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use crate::utils::lock_free::TracedMutex;

/// Wrapper for `Mutex<HashMap<K, V>>` with simplifies the API and prevents deadlocks
pub struct SyncHashMap<K, V>(TracedMutex<HashMap<K, V>>);

#[allow(dead_code, clippy::multiple_bound_locations)]
impl<K, V> SyncHashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self(TracedMutex::new(HashMap::new()))
    }

    pub fn len(&self) -> usize {
        self.0.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.lock().is_empty()
    }

    pub fn upsert(&self, key: K, value: V) -> Option<V> {
        self.0.lock().insert(key, value)
    }

    pub fn remove<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.0.lock().remove(key)
    }

    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.0.lock().contains_key(key)
    }

    pub fn get<Q: ?Sized, F, R>(&self, key: &Q, f: F) -> Option<R>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
        F: FnOnce(&mut V) -> R,
    {
        self.0.lock().get_mut(key).map(f)
    }

    /// If key does not exist, it will be created with default value
    pub fn get_or_default<Q, F, R>(&self, key: Q, f: F) -> R
    where
        V: Default,
        Q: Into<K>,
        F: FnOnce(&mut V) -> R,
    {
        f(self.0.lock().entry(key.into()).or_default())
    }

    /// If key does not exist, it will be created using the provided constructor function
    pub fn get_or_insert<Q, C, F, R>(&self, key: Q, constructor: C, f: F) -> R
    where
        Q: Into<K>,
        C: FnOnce() -> V,
        F: FnOnce(&mut V) -> R,
    {
        f(self.0.lock().entry(key.into()).or_insert_with(constructor))
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut((&K, &mut V)),
    {
        self.0.lock().iter_mut().for_each(f);
    }

    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.0.lock().retain(f);
    }

    pub fn clear(&self) {
        self.0.lock().clear();
    }

    pub fn any<F>(&self, f: F) -> bool
    where
        F: FnMut((&K, &V)) -> bool,
    {
        self.0.lock().iter().any(f)
    }

    pub fn take(&self) -> HashMap<K, V> {
        self.0.lock().drain().collect()
    }

    pub fn replace(&self, value: HashMap<K, V>) {
        *self.0.lock() = value;
    }
}

#[allow(dead_code)]
impl<K, V> SyncHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn to_hash_map(&self) -> HashMap<K, V> {
        self.0.lock().clone()
    }

    pub fn keys(&self) -> Vec<K> {
        self.0.lock().keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<V> {
        self.0.lock().values().cloned().collect()
    }
}

impl<K, V> From<HashMap<K, V>> for SyncHashMap<K, V>
where
    K: Eq + Hash,
{
    fn from(value: HashMap<K, V>) -> Self {
        Self(TracedMutex::new(value))
    }
}

impl<K, V> Default for SyncHashMap<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}
