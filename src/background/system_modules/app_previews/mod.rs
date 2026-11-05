//! Previews (thumbnails and edge colors) of the app windows. Its handlers live in the `apps`
//! module, as part of the app windows data.

mod application;
mod domain;

pub use self::{application::AppPreviewsManager, domain::PreviewsEvent};
