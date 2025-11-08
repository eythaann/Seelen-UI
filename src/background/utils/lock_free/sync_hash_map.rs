use std::collections::HashMap;
use std::hash::Hash;

use parking_lot::Mutex;

use crate::trace_lock;

/// Wrapper for `Mutex<HashMap<K, V>>` with simplifies the API and prevents deadlocks
pub struct SyncHashMap<K, V>(Mutex<HashMap<K, V>>);

#[allow(dead_code)]
impl<K, V> SyncHashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }

    pub fn len(&self) -> usize {
        trace_lock!(self.0).len()
    }

    pub fn is_empty(&self) -> bool {
        trace_lock!(self.0).is_empty()
    }

    pub fn upsert(&self, key: K, value: V) -> Option<V> {
        trace_lock!(self.0).insert(key, value)
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        trace_lock!(self.0).remove(key)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        trace_lock!(self.0).contains_key(key)
    }

    pub fn get<F, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&V) -> R,
    {
        trace_lock!(self.0).get(key).map(f)
    }

    pub fn get_mut<F, R>(&self, key: &K, f: F) -> Option<R>
    where
        F: FnOnce(&mut V) -> R,
    {
        trace_lock!(self.0).get_mut(key).map(f)
    }

    pub fn for_each<F>(&self, f: F)
    where
        F: FnMut((&K, &mut V)),
    {
        trace_lock!(self.0).iter_mut().for_each(f);
    }

    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        trace_lock!(self.0).retain(f);
    }

    pub fn clear(&self) {
        trace_lock!(self.0).clear();
    }

    pub fn any<F>(&self, f: F) -> bool
    where
        F: FnMut((&K, &V)) -> bool,
    {
        trace_lock!(self.0).iter().any(f)
    }
}

#[allow(dead_code)]
impl<K, V> SyncHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn to_hash_map(&self) -> HashMap<K, V> {
        trace_lock!(self.0).clone()
    }

    pub fn keys(&self) -> Vec<K> {
        trace_lock!(self.0).keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<V> {
        trace_lock!(self.0).values().cloned().collect()
    }
}

impl<K, V> From<HashMap<K, V>> for SyncHashMap<K, V>
where
    K: Eq + Hash,
{
    fn from(value: HashMap<K, V>) -> Self {
        Self(Mutex::new(value))
    }
}
