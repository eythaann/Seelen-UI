/// this macro slowness the build/check on dev so we split this
/// to this separated file to avoid slowness when editting main.rs
pub fn get_context() -> tauri::Context {
    tauri::generate_context!()
}
