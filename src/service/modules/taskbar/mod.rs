//! Native taskbar: hidden while Seelen UI replaces it, restored when it no longer does.

mod application;
mod domain;

pub use self::application::TaskbarManager;
