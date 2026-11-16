//! Previews (thumbnails and edge colors) of the app windows. Its handlers live in the `apps`
//! module, as part of the app windows data.

mod application;
mod domain;

pub use self::{application::AppPreviewsManager, domain::PreviewsEvent};

/// Release Windows compositor relationships and restore any temporary window on exit.
pub fn clear_native_previews() {
    if let Some(manager) = AppPreviewsManager::get_if_initialized() {
        manager.clear_native_previews();
    }
}
