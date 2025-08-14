use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use parking_lot::RwLock;
use seelen_core::system_state::MonitorId;

use super::Window;

static WINDOW_CACHE_DICT: LazyLock<Arc<RwLock<HashMap<isize, WindowCachedData>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowCachedData {
    pub hwnd: isize,
    pub monitor: MonitorId,
    pub maximized: bool,
    pub fullscreen: bool,
}

impl WindowCachedData {
    pub fn create_for(w: &Window) -> Self {
        WindowCachedData {
            hwnd: w.address(),
            monitor: w.monitor().stable_id().unwrap_or_default().into(),
            maximized: w.is_maximized(),
            fullscreen: w.is_fullscreen(),
        }
    }
}

impl Window {
    /// use this to ensure the cache is initialized, useful if we need information like monitor
    /// where the window was destroyed
    #[allow(dead_code)]
    pub fn init_cache(&self) {
        match WINDOW_CACHE_DICT.write().entry(self.address()) {
            std::collections::hash_map::Entry::Occupied(_) => {}
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(WindowCachedData::create_for(self));
            }
        }
    }

    pub fn get_cached_data(&self) -> WindowCachedData {
        if let Some(data) = WINDOW_CACHE_DICT.read().get(&self.address()) {
            return data.clone();
        }

        let data = WindowCachedData::create_for(self);
        self.set_cached_data(data.clone());
        data
    }

    pub fn set_cached_data(&self, data: WindowCachedData) {
        WINDOW_CACHE_DICT.write().insert(self.address(), data);
    }
}
