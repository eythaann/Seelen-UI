use tauri::{AppHandle, Wry};

pub struct SeelenShell {
    handle: AppHandle<Wry>,
}

impl SeelenShell {
    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self { handle }
    }
}
  