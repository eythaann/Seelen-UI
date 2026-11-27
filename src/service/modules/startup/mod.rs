//! Startup entry of the service: it launches the service (and so the app) on user logon.

mod application;
mod domain;

pub use self::application::StartupManager;
