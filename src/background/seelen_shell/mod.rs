use tauri::{AppHandle, Wry};

pub struct SeelenShell {
    #[allow(dead_code)]
    handle: AppHandle<Wry>,
}

impl SeelenShell {
    pub fn new(handle: AppHandle<Wry>) -> Self {
        Self { handle }
    }
}
