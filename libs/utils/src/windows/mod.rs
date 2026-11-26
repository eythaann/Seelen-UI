//! Windows API helpers (functions, wrappers, types) shared between multiple binaries (the
//! background process, the service, etc.).
//!
//! Only OS wrappers that don't depend on a specific binary belong here: errors are either
//! `windows::core::Result` or generic over the caller's error type, and anything that needs the
//! shared state of a specific binary stays in that binary, built on top of these helpers.

mod app_bar;
mod com;
mod foreground;
mod iterator;
mod string;

pub use app_bar::*;
pub use com::*;
pub use foreground::*;
pub use iterator::*;
pub use string::*;
