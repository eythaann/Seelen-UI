//! Positioning of the windows of other apps on behalf of Seelen UI, which can't move elevated
//! windows by itself.

mod application;
mod domain;

pub use self::application::WindowPositioningManager;
