//! Environment of the system: makes the installation folder (CLI) reachable through the `PATH`.

mod application;
mod domain;

pub use self::application::EnvironmentManager;
