//! Seelen UI app process: launch, kill and health monitoring (restarts it when it crashes).

mod application;
mod domain;

pub use self::application::AppManager;
