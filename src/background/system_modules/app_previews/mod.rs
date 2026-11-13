//! Previews (thumbnails and edge colors) of the app windows. Its handlers live in the `apps`
//! module, as part of the app windows data.

pub(crate) mod application;
mod domain;

pub use self::{application::AppPreviewsManager, domain::PreviewsEvent};

/// Release Windows compositor relationships and restore any temporary window on exit.
pub fn clear_native_previews() {
    #[cfg(windows)]
    {
        application::windows::live_previews::LivePreviewManager::instance().clear_all();
        application::windows::window_peek::WindowPeekManager::instance().cancel_all();
    }
}
