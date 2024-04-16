use tauri::{AppHandle, Wry};

pub struct SeelenBar {
    handle: AppHandle<Wry>,
}

impl SeelenBar {
    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self { handle }
    }
}
