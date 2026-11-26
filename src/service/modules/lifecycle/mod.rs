//! OS session lifecycle: suspend/resume and end of session (shutdown, logoff, package update).

mod application;
mod domain;

pub use self::application::LifecycleManager;
